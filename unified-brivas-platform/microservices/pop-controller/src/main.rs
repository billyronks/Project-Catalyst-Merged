//! PoP Controller - Local Autonomy and Federation Management
//!
//! Ensures each PoP can operate independently during network partitions
//! while maintaining eventual consistency when connectivity is restored.

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod autonomy;
mod federation;
mod peers;

pub use autonomy::{AutonomyController, AutonomyMode};
pub use federation::FederationConfig;
pub use peers::PeerMonitor;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!("Starting PoP Controller");

    let service = Arc::new(PopControllerService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct PopControllerService {
    pop_id: String,
    autonomy: AutonomyController,
    start_time: std::time::Instant,
}

impl PopControllerService {
    pub async fn new() -> Result<Self> {
        let pop_id = std::env::var("POP_ID").unwrap_or_else(|_| "unknown-pop".to_string());
        let db_url = std::env::var("LUMADB_URL")
            .unwrap_or_else(|_| "postgres://brivas:password@localhost:5432/brivas".to_string());

        let autonomy = AutonomyController::new(pop_id.clone(), db_url).await?;

        Ok(Self {
            pop_id,
            autonomy,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for PopControllerService {
    fn service_id(&self) -> &'static str {
        "pop-controller"
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
        let mode = self.autonomy.current_mode().await;
        ReadinessStatus {
            ready: true,
            dependencies: vec![brivas_core::DependencyStatus {
                name: format!("autonomy-mode:{:?}", mode),
                available: true,
                latency_ms: None,
            }],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!(pop_id = %self.pop_id, "Shutting down PoP Controller");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        info!(pop_id = %self.pop_id, bind = %http_bind, "Starting PoP Controller");

        // Start autonomy monitoring in background
        let autonomy = self.autonomy.clone();
        tokio::spawn(async move {
            if let Err(e) = autonomy.start().await {
                tracing::error!("Autonomy controller error: {}", e);
            }
        });

        // HTTP server for status
        let pop_id = self.pop_id.clone();
        let autonomy_status = self.autonomy.clone();
        
        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }))
            .route(
                "/status",
                axum::routing::get(move || {
                    let pop = pop_id.clone();
                    let auto = autonomy_status.clone();
                    async move {
                        let mode = auto.current_mode().await;
                        axum::Json(serde_json::json!({
                            "pop_id": pop,
                            "autonomy_mode": format!("{:?}", mode),
                            "healthy": true
                        }))
                    }
                }),
            );

        let listener = tokio::net::TcpListener::bind(&http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
