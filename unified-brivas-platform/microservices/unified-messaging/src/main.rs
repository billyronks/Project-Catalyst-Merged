//! Unified Messaging Hub - 16 Platform Integrations
//!
//! Supported platforms:
//! - Tier 1: WhatsApp, Facebook Messenger, Telegram, WeChat
//! - Tier 2: Snapchat, Signal, Viber, LINE, Discord
//! - Tier 3: iMessage, QQ, Zalo, KakaoTalk, Slack, Teams, Google Chat

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod adapters;
mod model;
mod service;

pub use adapters::{PlatformAdapter, PlatformCapabilities};
pub use model::{MessageContent, Platform, UnifiedMessage};
pub use service::MessagingHubService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("unified_messaging=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting Unified Messaging Hub");

    let service = Arc::new(MessagingHubServiceWrapper::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct MessagingHubServiceWrapper {
    inner: MessagingHubService,
    start_time: std::time::Instant,
}

impl MessagingHubServiceWrapper {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            inner: MessagingHubService::new().await?,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for MessagingHubServiceWrapper {
    fn service_id(&self) -> &'static str {
        "unified-messaging"
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
        let platforms = self.inner.active_platforms().await;
        ReadinessStatus {
            ready: !platforms.is_empty(),
            dependencies: platforms
                .into_iter()
                .map(|p| brivas_core::DependencyStatus {
                    name: format!("{:?}", p),
                    available: true,
                    latency_ms: None,
                })
                .collect(),
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Unified Messaging Hub");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        info!(bind = %http_bind, "Starting Messaging Hub servers");

        let inner = self.inner.clone();
        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }))
            .route(
                "/message/send",
                axum::routing::post(move |body| send_message(body, inner.clone())),
            );

        let listener = tokio::net::TcpListener::bind(&http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct SendMessageRequest {
    platform: String,
    recipient_id: String,
    content: model::MessageContent,
}

#[derive(serde::Serialize)]
struct SendMessageResponse {
    message_id: String,
    platform_message_id: String,
    status: String,
}

async fn send_message(
    axum::Json(req): axum::Json<SendMessageRequest>,
    hub: MessagingHubService,
) -> axum::Json<SendMessageResponse> {
    let platform = model::Platform::from_str(&req.platform);
    
    match hub.send_message(platform, &req.recipient_id, req.content).await {
        Ok((msg_id, platform_id)) => axum::Json(SendMessageResponse {
            message_id: msg_id,
            platform_message_id: platform_id,
            status: "sent".to_string(),
        }),
        Err(e) => axum::Json(SendMessageResponse {
            message_id: String::new(),
            platform_message_id: String::new(),
            status: format!("error: {}", e),
        }),
    }
}
