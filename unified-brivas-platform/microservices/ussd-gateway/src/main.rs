//! USSD Gateway Microservice
//!
//! Handles USSD session management, dynamic menu rendering,
//! and multi-operator integration (MTN, Airtel, Glo, 9Mobile).

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod menu;
mod operators;
mod session;

pub use menu::{MenuAction, MenuDefinition, MenuOption};
pub use operators::{Operator, OperatorRenderer};
pub use session::{UssdResponse, UssdSession, UssdSessionManager};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ussd_gateway=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting USSD Gateway microservice");

    let service = Arc::new(UssdGatewayService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct UssdGatewayService {
    config: UssdConfig,
    session_manager: UssdSessionManager,
    start_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct UssdConfig {
    pub http_bind: String,
    pub grpc_bind: String,
    pub lumadb_url: String,
    pub session_ttl_secs: u64,
}

impl UssdConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_bind: std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            grpc_bind: std::env::var("GRPC_BIND").unwrap_or_else(|_| "0.0.0.0:9090".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            session_ttl_secs: std::env::var("SESSION_TTL_SECS")
                .unwrap_or_else(|_| "180".to_string())
                .parse()
                .unwrap_or(180),
        })
    }
}

impl UssdGatewayService {
    pub async fn new() -> Result<Self> {
        let config = UssdConfig::from_env()?;
        let session_manager =
            UssdSessionManager::new(&config.lumadb_url, config.session_ttl_secs).await?;

        Ok(Self {
            config,
            session_manager,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for UssdGatewayService {
    fn service_id(&self) -> &'static str {
        "ussd-gateway"
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
            dependencies: vec![brivas_core::DependencyStatus {
                name: "lumadb".to_string(),
                available: true,
                latency_ms: Some(1),
            }],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down USSD Gateway");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            grpc = %self.config.grpc_bind,
            "Starting USSD Gateway servers"
        );

        let session_mgr = self.session_manager.clone();

        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }))
            .route(
                "/ussd/callback",
                axum::routing::post(move |body| handle_ussd_callback(body, session_mgr.clone())),
            );

        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// USSD callback request from operators/aggregators
#[derive(Debug, serde::Deserialize)]
pub struct UssdCallbackRequest {
    pub session_id: Option<String>,
    pub msisdn: String,
    pub service_code: String,
    pub input: String,
    pub operator: String,
    pub network_session_id: String,
}

/// USSD callback response
#[derive(Debug, serde::Serialize)]
pub struct UssdCallbackResponse {
    pub session_id: String,
    pub message: String,
    pub session_type: String, // "continue" or "end"
}

async fn handle_ussd_callback(
    axum::Json(req): axum::Json<UssdCallbackRequest>,
    session_mgr: UssdSessionManager,
) -> axum::Json<UssdCallbackResponse> {
    let operator = match req.operator.to_lowercase().as_str() {
        "mtn" => operators::Operator::Mtn,
        "airtel" => operators::Operator::Airtel,
        "glo" => operators::Operator::Glo,
        "9mobile" => operators::Operator::NineMobile,
        _ => operators::Operator::Unknown,
    };

    let (session, response) = match req.session_id {
        Some(sid) => {
            let mut session = session_mgr.get_or_create(&sid, &req.msisdn, operator).await;
            let resp = session.process_input(&req.input).await;
            (session, resp)
        }
        None => {
            let session = session_mgr.create(&req.msisdn, &req.service_code, operator).await;
            let resp = session.get_welcome_menu();
            (session, resp)
        }
    };

    axum::Json(UssdCallbackResponse {
        session_id: session.id.clone(),
        message: response.message,
        session_type: if response.end_session {
            "end".to_string()
        } else {
            "continue".to_string()
        },
    })
}
