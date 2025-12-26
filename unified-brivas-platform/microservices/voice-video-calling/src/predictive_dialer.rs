//! Predictive Dialer
//!
//! Outbound call center automation with predictive algorithm
//! to minimize agent idle time while keeping abandonment rate low.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use dashmap::DashMap;

use crate::VoiceIvrConfig;

/// Dialer session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialerSession {
    pub id: String,
    pub campaign_id: String,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub calls_placed: u64,
    pub calls_connected: u64,
    pub calls_abandoned: u64,
    pub avg_wait_time_ms: u64,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Paused,
    Stopped,
}

/// Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub extension: String,
    pub status: AgentStatus,
    pub current_call_id: Option<String>,
    pub calls_handled: u64,
    pub avg_handle_time_seconds: f64,
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Available,
    OnCall,
    WrapUp,
    Away,
    Offline,
}

/// Dialer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialerStats {
    pub session_id: String,
    pub calls_placed: u64,
    pub calls_connected: u64,
    pub calls_abandoned: u64,
    pub connect_rate: f64,
    pub abandonment_rate: f64,
    pub avg_wait_time_ms: u64,
    pub active_agents: u32,
    pub available_agents: u32,
}

/// Predictive algorithm config
#[derive(Debug, Clone)]
pub struct PredictiveConfig {
    pub target_abandonment_rate: f64,  // e.g., 0.03 = 3%
    pub avg_handle_time_seconds: f64,
    pub dial_factor: f64,  // Initial calls per agent
}

impl Default for PredictiveConfig {
    fn default() -> Self {
        Self {
            target_abandonment_rate: 0.03,
            avg_handle_time_seconds: 180.0,
            dial_factor: 1.2,
        }
    }
}

/// Predictive Dialer
pub struct PredictiveDialer {
    #[allow(dead_code)]
    config: VoiceIvrConfig,
    predictive_config: PredictiveConfig,
    sessions: Arc<DashMap<String, DialerSession>>,
    agents: Arc<DashMap<String, Agent>>,
}

impl PredictiveDialer {
    pub async fn new(config: &VoiceIvrConfig) -> brivas_core::Result<Self> {
        Ok(Self {
            config: config.clone(),
            predictive_config: PredictiveConfig::default(),
            sessions: Arc::new(DashMap::new()),
            agents: Arc::new(DashMap::new()),
        })
    }

    /// Start a dialer session
    pub async fn start_session(&self, campaign_id: &str) -> Result<DialerSession, DialerError> {
        let session = DialerSession {
            id: uuid::Uuid::new_v4().to_string(),
            campaign_id: campaign_id.to_string(),
            status: SessionStatus::Active,
            started_at: Utc::now(),
            calls_placed: 0,
            calls_connected: 0,
            calls_abandoned: 0,
            avg_wait_time_ms: 0,
        };

        self.sessions.insert(session.id.clone(), session.clone());

        // Start predictive loop
        let sessions = self.sessions.clone();
        let agents = self.agents.clone();
        let sid = session.id.clone();
        let pc = self.predictive_config.clone();

        tokio::spawn(async move {
            Self::run_predictive_loop(&sessions, &agents, &sid, &pc).await;
        });

        tracing::info!(session_id = %session.id, "Dialer session started");

        Ok(session)
    }

    /// Run the predictive dialing loop
    async fn run_predictive_loop(
        sessions: &DashMap<String, DialerSession>,
        agents: &DashMap<String, Agent>,
        session_id: &str,
        config: &PredictiveConfig,
    ) {
        loop {
            // Check session status
            let session = match sessions.get(session_id) {
                Some(s) if s.status == SessionStatus::Active => s.clone(),
                _ => break,
            };

            // Count available agents
            let available_agents: usize = agents
                .iter()
                .filter(|a| a.status == AgentStatus::Available)
                .count();

            if available_agents == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            // Calculate calls to place using predictive algorithm
            let calls_to_place = Self::calculate_calls_to_place(
                available_agents,
                &session,
                config,
            );

            tracing::debug!(
                session_id = %session_id,
                available_agents = available_agents,
                calls_to_place = calls_to_place,
                "Predictive calculation"
            );

            // Place calls
            for _ in 0..calls_to_place {
                // TODO: Get next recipient and place call
                if let Some(mut s) = sessions.get_mut(session_id) {
                    s.calls_placed += 1;
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    /// Calculate optimal number of calls to place
    fn calculate_calls_to_place(
        available_agents: usize,
        session: &DialerSession,
        config: &PredictiveConfig,
    ) -> usize {
        // Historical connect rate
        let connect_rate = if session.calls_placed > 0 {
            session.calls_connected as f64 / session.calls_placed as f64
        } else {
            0.3 // Default assumption
        };

        // Base calculation: agents * dial_factor / connect_rate
        let calls_needed = (available_agents as f64 * config.dial_factor / connect_rate).ceil() as usize;

        // Cap based on abandonment rate
        let current_abandonment = if session.calls_connected > 0 {
            session.calls_abandoned as f64 / session.calls_connected as f64
        } else {
            0.0
        };

        // Reduce if abandonment is too high
        if current_abandonment > config.target_abandonment_rate {
            (calls_needed as f64 * 0.8).ceil() as usize
        } else {
            calls_needed
        }
    }

    /// Register an agent
    pub fn register_agent(&self, id: &str, extension: &str) {
        let agent = Agent {
            id: id.to_string(),
            extension: extension.to_string(),
            status: AgentStatus::Available,
            current_call_id: None,
            calls_handled: 0,
            avg_handle_time_seconds: 0.0,
        };
        self.agents.insert(id.to_string(), agent);
    }

    /// Update agent status
    pub fn update_agent_status(&self, agent_id: &str, status: AgentStatus) -> Result<(), DialerError> {
        let mut agent = self.agents
            .get_mut(agent_id)
            .ok_or(DialerError::AgentNotFound)?;
        agent.status = status;
        Ok(())
    }

    /// Stop a session
    pub async fn stop_session(&self, session_id: &str) -> Result<DialerSession, DialerError> {
        let mut session = self.sessions
            .get_mut(session_id)
            .ok_or(DialerError::SessionNotFound)?;
        session.status = SessionStatus::Stopped;
        Ok(session.clone())
    }

    /// Get session stats
    pub fn get_stats(&self, session_id: &str) -> Result<DialerStats, DialerError> {
        let session = self.sessions
            .get(session_id)
            .ok_or(DialerError::SessionNotFound)?;

        let connect_rate = if session.calls_placed > 0 {
            session.calls_connected as f64 / session.calls_placed as f64
        } else {
            0.0
        };

        let abandonment_rate = if session.calls_connected > 0 {
            session.calls_abandoned as f64 / session.calls_connected as f64
        } else {
            0.0
        };

        let active_agents = self.agents.iter().filter(|a| a.status == AgentStatus::OnCall).count() as u32;
        let available_agents = self.agents.iter().filter(|a| a.status == AgentStatus::Available).count() as u32;

        Ok(DialerStats {
            session_id: session_id.to_string(),
            calls_placed: session.calls_placed,
            calls_connected: session.calls_connected,
            calls_abandoned: session.calls_abandoned,
            connect_rate,
            abandonment_rate,
            avg_wait_time_ms: session.avg_wait_time_ms,
            active_agents,
            available_agents,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DialerError {
    #[error("Session not found")]
    SessionNotFound,

    #[error("Agent not found")]
    AgentNotFound,

    #[error("No agents available")]
    NoAgentsAvailable,
}
