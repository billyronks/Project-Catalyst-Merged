//! Temporal Worker Microservice
//!
//! Orchestrates complex, long-running workflows for the VAS platform:
//! - Service provisioning workflows
//! - Call routing with failover
//! - Billing and rating sagas
//! - Fraud detection pipelines

mod config;
mod workflows;
mod activities;

use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("Starting Temporal Worker microservice");

    // Load configuration
    let config = Config::from_env()?;

    info!("Temporal server: {}:{}", config.temporal_host, config.temporal_port);
    info!("Namespace: {}", config.temporal_namespace);
    info!("Task queue: {}", config.task_queue);

    // Initialize database pool for activities
    let db = brivas_lumadb::LumaDbPool::new(&config.database_url).await?;
    let db = Arc::new(db);

    // In production, this would:
    // 1. Create a Temporal client connection
    // 2. Register workflows and activities
    // 3. Start a worker polling the task queue

    // For now, we start a simple health check server
    let app = axum::Router::new()
        .route("/health", axum::routing::get(health))
        .route("/ready", axum::routing::get(ready));

    let addr = config.bind_address();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Temporal worker health server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "temporal-worker",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn ready() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "ready": true
    }))
}
