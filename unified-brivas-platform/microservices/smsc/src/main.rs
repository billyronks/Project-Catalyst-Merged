//! SMSC - High-Performance SMS Center Microservice
//!
//! Capabilities:
//! - 100,000+ TPS throughput
//! - SMPP v3.4/5.0 server
//! - Intelligent message routing
//! - Delivery report handling
//! - Message queuing via LumaDB Streams

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod queue;
mod routing;
mod smpp;

pub use queue::MessageQueue;
pub use routing::MessageRouter;
pub use smpp::SmppServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("smsc=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting SMSC microservice");

    let service = Arc::new(SmscService::new().await?);
    MicroserviceRuntime::run(service).await
}

/// SMSC Service implementation
pub struct SmscService {
    config: SmscConfig,
    message_queue: MessageQueue,
    router: MessageRouter,
    smpp_server: SmppServer,
    start_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct SmscConfig {
    pub smpp_bind_address: String,
    pub http_bind_address: String,
    pub grpc_bind_address: String,
    pub lumadb_url: String,
    pub target_tps: u32,
    pub max_connections: usize,
}

impl SmscConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            smpp_bind_address: std::env::var("SMPP_BIND")
                .unwrap_or_else(|_| "0.0.0.0:2775".to_string()),
            http_bind_address: std::env::var("HTTP_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            grpc_bind_address: std::env::var("GRPC_BIND")
                .unwrap_or_else(|_| "0.0.0.0:9090".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            target_tps: std::env::var("TARGET_TPS")
                .unwrap_or_else(|_| "100000".to_string())
                .parse()
                .unwrap_or(100000),
            max_connections: std::env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
        })
    }
}

impl SmscService {
    pub async fn new() -> Result<Self> {
        let config = SmscConfig::from_env()?;

        info!(
            target_tps = config.target_tps,
            max_connections = config.max_connections,
            "Initializing SMSC"
        );

        let message_queue = MessageQueue::new(&config.lumadb_url).await?;
        let router = MessageRouter::new(&config.lumadb_url).await?;
        let smpp_server = SmppServer::new(&config.smpp_bind_address, config.max_connections);

        Ok(Self {
            config,
            message_queue,
            router,
            smpp_server,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for SmscService {
    fn service_id(&self) -> &'static str {
        "smsc"
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
                    available: self.message_queue.is_healthy().await,
                    latency_ms: Some(1),
                },
                brivas_core::DependencyStatus {
                    name: "smpp_server".to_string(),
                    available: self.smpp_server.is_running(),
                    latency_ms: None,
                },
            ],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down SMSC");
        self.smpp_server.stop().await;
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            smpp_addr = %self.config.smpp_bind_address,
            http_addr = %self.config.http_bind_address,
            "Starting SMSC servers"
        );

        // Start SMPP server in background
        let smpp = self.smpp_server.clone();
        let queue = self.message_queue.clone();
        let router = self.router.clone();

        tokio::spawn(async move {
            if let Err(e) = smpp.run(queue, router).await {
                tracing::error!("SMPP server error: {}", e);
            }
        });

        // Start HTTP server for health/metrics
        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }));

        let listener = tokio::net::TcpListener::bind(&self.config.http_bind_address).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
