//! Instant Messaging Microservice
//!
//! Enterprise-grade real-time messaging with:
//! - End-to-end encryption (Signal Protocol)
//! - Presence and typing indicators
//! - Group chats and broadcast channels
//! - File sharing with CDN delivery
//! - Multi-device synchronization

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod config;
mod api;
mod domain;
mod infrastructure;
mod encryption;
mod handlers;

pub use config::ImConfig;
pub use domain::{Conversation, Message};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("instant_messaging=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting Instant Messaging microservice");

    let service = Arc::new(InstantMessagingService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct InstantMessagingService {
    config: ImConfig,
    conversation_store: infrastructure::ConversationStore,
    message_store: infrastructure::MessageStore,
    presence_manager: infrastructure::PresenceManager,
    encryption_service: encryption::EncryptionService,
    start_time: std::time::Instant,
}

impl InstantMessagingService {
    pub async fn new() -> Result<Self> {
        let config = ImConfig::from_env()?;
        let conversation_store = infrastructure::ConversationStore::new(&config.lumadb_url).await?;
        let message_store = infrastructure::MessageStore::new(&config.lumadb_url).await?;
        let presence_manager = infrastructure::PresenceManager::new();
        let encryption_service = encryption::EncryptionService::new();

        Ok(Self {
            config,
            conversation_store,
            message_store,
            presence_manager,
            encryption_service,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for InstantMessagingService {
    fn service_id(&self) -> &'static str {
        "instant-messaging"
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
            ],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Instant Messaging service");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            ws = %self.config.ws_bind,
            "Starting Instant Messaging servers"
        );

        let app = api::create_router(self);
        
        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
