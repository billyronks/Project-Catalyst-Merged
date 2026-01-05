//! AIOps Engine
//!
//! Autonomous IT Operations platform for:
//! - Anomaly detection using LumaDB TSDB
//! - Automated remediation via playbooks
//! - Cross-service health correlation
//! - SMPP bind disconnect recovery
//! - GitOps integration for configuration management

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use brivas_lumadb::{LumaDbPool, PoolConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod anomaly;
mod config;
mod metrics;
mod playbook;
mod remediation;

pub use config::AiOpsConfig;
use anomaly::AnomalyDetector;
use playbook::PlaybookExecutor;
use remediation::RemediationOrchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("aiops_engine=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting AIOps Engine");

    let service = Arc::new(AiOpsService::new().await?);
    MicroserviceRuntime::run(service).await
}

/// AIOps service state
pub struct AiOpsService {
    config: AiOpsConfig,
    pool: LumaDbPool,
    anomaly_detector: Arc<AnomalyDetector>,
    remediation_orchestrator: Arc<RemediationOrchestrator>,
    playbook_executor: Arc<PlaybookExecutor>,
    start_time: std::time::Instant,
    active_incidents: Arc<RwLock<Vec<Incident>>>,
}

/// Active incident being tracked
#[derive(Debug, Clone)]
pub struct Incident {
    pub id: String,
    pub source: String,
    pub severity: Severity,
    pub description: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub status: IncidentStatus,
    pub playbook_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IncidentStatus {
    Detected,
    Acknowledged,
    Remediating,
    Resolved,
    Escalated,
}

impl AiOpsService {
    pub async fn new() -> Result<Self> {
        let config = AiOpsConfig::from_env()?;
        
        // Create LumaDB connection pool
        let pool_config = PoolConfig {
            url: config.lumadb_url.clone(),
            max_size: 16,
            min_idle: Some(2),
        };
        let pool = LumaDbPool::new(pool_config).await
            .map_err(|e| brivas_core::BrivasError::Database(e.to_string()))?;
        
        // Initialize components
        let anomaly_detector = Arc::new(AnomalyDetector::new(pool.clone(), &config));
        let playbook_executor = Arc::new(PlaybookExecutor::new(&config.playbooks_dir));
        let remediation_orchestrator = Arc::new(RemediationOrchestrator::new(
            playbook_executor.clone(),
            pool.clone(),
        ));

        Ok(Self {
            config,
            pool,
            anomaly_detector,
            remediation_orchestrator,
            playbook_executor,
            start_time: std::time::Instant::now(),
            active_incidents: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Start the anomaly detection loop
    async fn run_detection_loop(&self) {
        let detector = self.anomaly_detector.clone();
        let orchestrator = self.remediation_orchestrator.clone();
        let incidents = self.active_incidents.clone();
        let check_interval = self.config.check_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(check_interval)
            );
            
            loop {
                interval.tick().await;
                
                // Run anomaly detection
                if let Ok(anomalies) = detector.detect_all().await {
                    for anomaly in anomalies {
                        let incident = Incident {
                            id: uuid::Uuid::new_v4().to_string(),
                            source: anomaly.source.clone(),
                            severity: anomaly.severity,
                            description: anomaly.description.clone(),
                            detected_at: chrono::Utc::now(),
                            status: IncidentStatus::Detected,
                            playbook_id: anomaly.recommended_playbook.clone(),
                        };
                        
                        info!(
                            incident_id = %incident.id,
                            source = %incident.source,
                            severity = ?incident.severity,
                            "Anomaly detected"
                        );
                        
                        // Attempt auto-remediation
                        if let Some(playbook_id) = &incident.playbook_id {
                            let mut incident_mut = incident.clone();
                            incident_mut.status = IncidentStatus::Remediating;
                            
                            if orchestrator.execute(playbook_id, &anomaly.context).await.is_ok() {
                                incident_mut.status = IncidentStatus::Resolved;
                                info!(incident_id = %incident.id, "Auto-remediation successful");
                            } else {
                                incident_mut.status = IncidentStatus::Escalated;
                                info!(incident_id = %incident.id, "Escalating to on-call");
                            }
                            
                            incidents.write().await.push(incident_mut);
                        } else {
                            incidents.write().await.push(incident);
                        }
                    }
                }
            }
        });
    }
}

#[async_trait::async_trait]
impl BrivasService for AiOpsService {
    fn service_id(&self) -> &'static str {
        "aiops-engine"
    }

    async fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            service_id: self.service_id().to_string(),
            version: self.version().to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    async fn ready(&self) -> ReadinessStatus {
        let db_healthy = self.pool.is_healthy().await;
        ReadinessStatus {
            ready: db_healthy,
            dependencies: vec![brivas_core::DependencyStatus {
                name: "lumadb".to_string(),
                available: db_healthy,
                latency_ms: Some(1),
            }],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down AIOps Engine");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            check_interval = %self.config.check_interval_secs,
            "Starting AIOps Engine"
        );
        
        // Start anomaly detection loop
        self.run_detection_loop().await;
        
        // Create HTTP routes
        let incidents = self.active_incidents.clone();
        let detector = self.anomaly_detector.clone();
        let orchestrator = self.remediation_orchestrator.clone();
        
        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }))
            .route("/api/v1/incidents", axum::routing::get({
                let incidents = incidents.clone();
                move || {
                    let incidents = incidents.clone();
                    async move {
                        let list = incidents.read().await;
                        axum::Json(serde_json::json!({
                            "incidents": list.len(),
                            "active": list.iter().filter(|i| i.status != IncidentStatus::Resolved).count()
                        }))
                    }
                }
            }))
            .route("/api/v1/detect", axum::routing::post({
                let detector = detector.clone();
                move || {
                    let detector = detector.clone();
                    async move {
                        match detector.detect_all().await {
                            Ok(anomalies) => axum::Json(serde_json::json!({
                                "anomalies": anomalies.len()
                            })),
                            Err(e) => axum::Json(serde_json::json!({
                                "error": e.to_string()
                            })),
                        }
                    }
                }
            }))
            .route("/api/v1/playbooks/:id/execute", axum::routing::post({
                let orchestrator = orchestrator.clone();
                move |axum::extract::Path(id): axum::extract::Path<String>| {
                    let orchestrator = orchestrator.clone();
                    async move {
                        match orchestrator.execute(&id, &serde_json::json!({})).await {
                            Ok(_) => axum::Json(serde_json::json!({ "status": "executed" })),
                            Err(e) => axum::Json(serde_json::json!({ "error": e.to_string() })),
                        }
                    }
                }
            }));

        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
