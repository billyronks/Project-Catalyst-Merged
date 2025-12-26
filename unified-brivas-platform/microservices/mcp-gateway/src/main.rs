//! MCP Gateway
//!
//! Model Context Protocol server for AI/LLM integration:
//! - Tools: Actions LLMs can invoke (send_sms, create_campaign, etc.)
//! - Resources: Data LLMs can read (conversation context, analytics)
//! - Prompts: Pre-built templates for common tasks
//!
//! Supports both stdio and SSE transports for Claude Desktop, Cursor, etc.

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod config;
mod server;
mod tools;
mod resources;
mod prompts;

pub use config::McpConfig;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mcp_gateway=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting MCP Gateway");

    let service = Arc::new(McpGatewayService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct McpGatewayService {
    config: McpConfig,
    mcp_server: server::McpServer,
    start_time: std::time::Instant,
}

impl McpGatewayService {
    pub async fn new() -> Result<Self> {
        let config = McpConfig::from_env()?;
        let mcp_server = server::McpServer::new(&config).await?;

        Ok(Self {
            config,
            mcp_server,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for McpGatewayService {
    fn service_id(&self) -> &'static str {
        "mcp-gateway"
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
        info!("Shutting down MCP Gateway");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            "Starting MCP Gateway server"
        );

        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }))
            .route("/mcp/v1/sse", axum::routing::get(server::sse_handler));
        
        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
