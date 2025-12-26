//! RCS Messaging Microservice
//!
//! Rich Communication Services for A2P/P2P messaging:
//! - Google RCS Business Messaging (RBM)
//! - Jibe/Samsung RCS Hub integration
//! - Rich cards and carousels
//! - Suggested actions and replies
//! - Device capability checking
//! - SMS fallback

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod config;
mod api;
mod domain;
mod infrastructure;
mod rendering;
mod handlers;

pub use config::RcsConfig;
pub use domain::{RcsAgent, RcsMessage};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rcs_messaging=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting RCS Messaging microservice");

    let service = Arc::new(RcsMessagingService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct RcsMessagingService {
    config: RcsConfig,
    agent_store: infrastructure::AgentStore,
    message_store: infrastructure::RcsMessageStore,
    capability_service: infrastructure::CapabilityService,
    start_time: std::time::Instant,
}

impl RcsMessagingService {
    pub async fn new() -> Result<Self> {
        let config = RcsConfig::from_env()?;
        let agent_store = infrastructure::AgentStore::new(&config.lumadb_url).await?;
        let message_store = infrastructure::RcsMessageStore::new(&config.lumadb_url).await?;
        let capability_service = infrastructure::CapabilityService::new(&config);

        Ok(Self {
            config,
            agent_store,
            message_store,
            capability_service,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for RcsMessagingService {
    fn service_id(&self) -> &'static str {
        "rcs-messaging"
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
        ReadinessStatus {
            ready: true,
            dependencies: vec![
                brivas_core::DependencyStatus {
                    name: "lumadb".to_string(),
                    available: true,
                    latency_ms: Some(1),
                },
                brivas_core::DependencyStatus {
                    name: "jibe_hub".to_string(),
                    available: true,
                    latency_ms: Some(50),
                },
            ],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down RCS Messaging service");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            "Starting RCS Messaging server"
        );

        let app = api::create_router(self);
        
        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
