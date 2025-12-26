//! Service infrastructure for all microservices

#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

use crate::config::ServiceConfig;
use crate::error::Result;

/// Health status for liveness probes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub service_id: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Readiness status for readiness probes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessStatus {
    pub ready: bool,
    pub dependencies: Vec<DependencyStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub available: bool,
    pub latency_ms: Option<u64>,
}

/// Standard trait all microservices must implement
#[async_trait]
pub trait BrivasService: Send + Sync + 'static {
    /// Service identifier (e.g., "ussd-gateway", "smsc")
    fn service_id(&self) -> &'static str;

    /// Service version
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Health check - is the service alive?
    async fn health(&self) -> HealthStatus;

    /// Readiness check - are all dependencies available?
    async fn ready(&self) -> ReadinessStatus;

    /// Graceful shutdown
    async fn shutdown(&self) -> Result<()>;

    /// Start the service (gRPC, HTTP servers, etc.)
    async fn start(&self) -> Result<()>;
}

/// Standard microservice runtime bootstrap
pub struct MicroserviceRuntime {
    config: ServiceConfig,
    start_time: std::time::Instant,
}

impl MicroserviceRuntime {
    /// Create new runtime from environment
    pub fn new() -> Result<Self> {
        let config = ServiceConfig::from_env()?;
        Ok(Self {
            config,
            start_time: std::time::Instant::now(),
        })
    }

    /// Run a microservice with standard lifecycle management
    pub async fn run<S: BrivasService>(service: Arc<S>) -> Result<()> {
        let runtime = Self::new()?;

        info!(
            service_id = service.service_id(),
            version = service.version(),
            "Starting microservice"
        );

        // Start the service
        let service_clone = service.clone();
        let service_handle = tokio::spawn(async move {
            if let Err(e) = service_clone.start().await {
                tracing::error!("Service error: {}", e);
            }
        });

        // Wait for shutdown signal
        Self::wait_for_shutdown().await;

        info!("Shutdown signal received, gracefully stopping...");

        // Graceful shutdown
        if let Err(e) = service.shutdown().await {
            warn!("Error during shutdown: {}", e);
        }

        service_handle.abort();

        info!(
            uptime_seconds = runtime.start_time.elapsed().as_secs(),
            "Microservice stopped"
        );

        Ok(())
    }

    async fn wait_for_shutdown() {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to listen for SIGTERM")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }
}

impl Default for MicroserviceRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create runtime")
    }
}
