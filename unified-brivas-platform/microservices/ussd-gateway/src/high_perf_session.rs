//! High-Performance USSD Session Manager
//!
//! Handles 100K+ concurrent sessions with:
//! - Sub-millisecond session lookups via DashMap
//! - LumaDB persistence for durability
//! - Automatic session timeout and cleanup
//! - Multi-operator support (MTN, Airtel, Glo, 9Mobile)

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// High-performance session manager
#[derive(Clone)]
pub struct SessionManager {
    /// In-memory session store (100K+ concurrent)
    sessions: Arc<DashMap<String, UssdSession>>,
    /// Database pool for persistence
    db: Arc<tokio_postgres::Client>,
    /// Configuration
    config: SessionConfig,
    /// Metrics
    metrics: Arc<RwLock<SessionMetrics>>,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub default_ttl_secs: u64,
    pub max_sessions: usize,
    pub cleanup_interval_secs: u64,
    pub persist_to_db: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_ttl_secs: 180,
            max_sessions: 100_000,
            cleanup_interval_secs: 30,
            persist_to_db: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UssdSession {
    pub id: String,
    pub msisdn: String,
    pub service_code: String,
    pub operator: Operator,
    pub current_menu: String,
    pub menu_stack: Vec<String>,
    pub data: std::collections::HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub state: SessionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Active,
    WaitingInput,
    Processing,
    Completed,
    TimedOut,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    Mtn,
    Airtel,
    Glo,
    NineMobile,
    Safaricom,
    Vodacom,
    Unknown,
}

impl Operator {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mtn" => Self::Mtn,
            "airtel" => Self::Airtel,
            "glo" => Self::Glo,
            "9mobile" | "etisalat" => Self::NineMobile,
            "safaricom" => Self::Safaricom,
            "vodacom" => Self::Vodacom,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Default)]
pub struct SessionMetrics {
    pub active_sessions: u64,
    pub total_created: u64,
    pub total_completed: u64,
    pub total_timed_out: u64,
    pub avg_duration_secs: f64,
}

impl SessionManager {
    pub async fn new(db_url: &str, config: SessionConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (client, connection) = tokio_postgres::connect(db_url, tokio_postgres::NoTls).await?;
        
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("Database connection error: {}", e);
            }
        });

        let manager = Self {
            sessions: Arc::new(DashMap::new()),
            db: Arc::new(client),
            config,
            metrics: Arc::new(RwLock::new(SessionMetrics::default())),
        };

        // Start cleanup task
        let cleanup_manager = manager.clone();
        tokio::spawn(async move {
            cleanup_manager.cleanup_loop().await;
        });

        Ok(manager)
    }

    /// Create a new session
    pub async fn create(
        &self,
        msisdn: &str,
        service_code: &str,
        operator: Operator,
    ) -> UssdSession {
        let now = Utc::now();
        let session_id = format!("{}_{}", msisdn, now.timestamp_millis());
        let ttl = Duration::seconds(self.config.default_ttl_secs as i64);

        let session = UssdSession {
            id: session_id.clone(),
            msisdn: msisdn.to_string(),
            service_code: service_code.to_string(),
            operator,
            current_menu: "main".to_string(),
            menu_stack: vec!["main".to_string()],
            data: std::collections::HashMap::new(),
            created_at: now,
            last_activity: now,
            expires_at: now + ttl,
            state: SessionState::Active,
        };

        self.sessions.insert(session_id.clone(), session.clone());

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.active_sessions += 1;
            metrics.total_created += 1;
        }

        // Persist to database
        if self.config.persist_to_db {
            let db = self.db.clone();
            let session_clone = session.clone();
            tokio::spawn(async move {
                Self::persist_session(&db, &session_clone).await;
            });
        }

        info!(session_id, msisdn, ?operator, "Session created");
        session
    }

    /// Get or create session
    pub async fn get_or_create(
        &self,
        session_id: &str,
        msisdn: &str,
        operator: Operator,
    ) -> UssdSession {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.last_activity = Utc::now();
            return session.clone();
        }

        // Try to load from database
        if let Some(session) = self.load_from_db(session_id).await {
            self.sessions.insert(session_id.to_string(), session.clone());
            return session;
        }

        // Create new
        self.create(msisdn, "*123#", operator).await
    }

    /// Update session
    pub async fn update(&self, session: UssdSession) {
        let session_id = session.id.clone();
        self.sessions.insert(session_id.clone(), session.clone());

        if self.config.persist_to_db {
            let db = self.db.clone();
            tokio::spawn(async move {
                Self::persist_session(&db, &session).await;
            });
        }
    }

    /// Complete session
    pub async fn complete(&self, session_id: &str) {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.state = SessionState::Completed;
            let duration = (Utc::now() - session.created_at).num_seconds();

            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.active_sessions = metrics.active_sessions.saturating_sub(1);
                metrics.total_completed += 1;
                // Update running average
                let total = metrics.total_completed as f64;
                metrics.avg_duration_secs = 
                    (metrics.avg_duration_secs * (total - 1.0) + duration as f64) / total;
            }
        }

        // Remove after short delay
        let sessions = self.sessions.clone();
        let sid = session_id.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            sessions.remove(&sid);
        });
    }

    /// Navigate to menu
    pub fn navigate(&self, session_id: &str, menu_id: &str) {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.menu_stack.push(menu_id.to_string());
            session.current_menu = menu_id.to_string();
            session.last_activity = Utc::now();
        }
    }

    /// Go back in menu
    pub fn go_back(&self, session_id: &str) -> Option<String> {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            if session.menu_stack.len() > 1 {
                session.menu_stack.pop();
                session.current_menu = session.menu_stack.last().cloned().unwrap_or("main".into());
                session.last_activity = Utc::now();
                return Some(session.current_menu.clone());
            }
        }
        None
    }

    /// Store session data
    pub fn set_data(&self, session_id: &str, key: &str, value: &str) {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.data.insert(key.to_string(), value.to_string());
            session.last_activity = Utc::now();
        }
    }

    /// Get session data
    pub fn get_data(&self, session_id: &str, key: &str) -> Option<String> {
        self.sessions
            .get(session_id)
            .and_then(|s| s.data.get(key).cloned())
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> SessionMetrics {
        let metrics = self.metrics.read().await;
        SessionMetrics {
            active_sessions: self.sessions.len() as u64,
            ..*metrics
        }
    }

    async fn persist_session(db: &tokio_postgres::Client, session: &UssdSession) {
        let data_json = serde_json::to_value(&session.data).unwrap_or_default();
        let menu_stack_json = serde_json::to_value(&session.menu_stack).unwrap_or_default();

        db.execute(
            "INSERT INTO ussd_sessions (id, msisdn, service_code, operator, current_menu, menu_stack, data, created_at, last_activity, expires_at, state)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT (id) DO UPDATE SET
             current_menu = $5, menu_stack = $6, data = $7, last_activity = $9, state = $11",
            &[
                &session.id,
                &session.msisdn,
                &session.service_code,
                &format!("{:?}", session.operator),
                &session.current_menu,
                &menu_stack_json,
                &data_json,
                &session.created_at,
                &session.last_activity,
                &session.expires_at,
                &format!("{:?}", session.state),
            ],
        ).await.ok();
    }

    async fn load_from_db(&self, session_id: &str) -> Option<UssdSession> {
        let row = self.db.query_opt(
            "SELECT id, msisdn, service_code, operator, current_menu, menu_stack, data, created_at, last_activity, expires_at, state
             FROM ussd_sessions WHERE id = $1",
            &[&session_id],
        ).await.ok()??;

        Some(UssdSession {
            id: row.get(0),
            msisdn: row.get(1),
            service_code: row.get(2),
            operator: Operator::from_str(row.get::<_, String>(3).as_str()),
            current_menu: row.get(4),
            menu_stack: serde_json::from_value(row.get(5)).unwrap_or_default(),
            data: serde_json::from_value(row.get(6)).unwrap_or_default(),
            created_at: row.get(7),
            last_activity: row.get(8),
            expires_at: row.get(9),
            state: SessionState::Active,
        })
    }

    async fn cleanup_loop(&self) {
        let interval = tokio::time::Duration::from_secs(self.config.cleanup_interval_secs);
        let mut ticker = tokio::time::interval(interval);

        loop {
            ticker.tick().await;
            self.cleanup_expired().await;
        }
    }

    async fn cleanup_expired(&self) {
        let now = Utc::now();
        let mut expired = 0u64;

        self.sessions.retain(|_, session| {
            if session.expires_at < now {
                expired += 1;
                false
            } else {
                true
            }
        });

        if expired > 0 {
            let mut metrics = self.metrics.write().await;
            metrics.active_sessions = metrics.active_sessions.saturating_sub(expired);
            metrics.total_timed_out += expired;
            debug!(expired, "Cleaned up expired sessions");
        }
    }
}
