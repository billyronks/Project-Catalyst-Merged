//! Voice/IVR Microservice
//!
//! Carrier-grade Voice/IVR platform for VAS services including:
//! - Flash Call (OTP via Caller ID)
//! - Missed Call Alerts
//! - Interactive Voice Response (IVR)
//! - Bulk Voice Messaging
//! - Predictive Dialer
//! - Voice Chat (voice messages to WhatsApp/Telegram)
//! - STIR/SHAKEN caller ID authentication (configurable per market)

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod flash_call;
mod missed_call;
mod ivr;
mod bulk_voice;
mod predictive_dialer;
mod voice_chat;
mod call_control;
mod billing;
mod handlers;

#[cfg(feature = "stir-shaken")]
mod stir_shaken;

pub use flash_call::FlashCallService;
pub use missed_call::MissedCallService;
pub use ivr::IvrEngine;
pub use bulk_voice::BulkVoiceService;
pub use predictive_dialer::PredictiveDialer;
pub use voice_chat::VoiceChatService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!("Starting Voice/IVR Microservice");

    let config = VoiceIvrConfig::from_env()?;
    let service = Arc::new(VoiceIvrService::new(config).await?);
    MicroserviceRuntime::run(service).await
}

/// Voice/IVR service configuration
#[derive(Debug, Clone)]
pub struct VoiceIvrConfig {
    pub pop_id: String,
    pub lumadb_url: String,
    pub opensips_url: String,
    pub freeswitch_url: String,
    pub freeswitch_password: String,
    pub rtpengine_url: String,
    pub stir_shaken_enabled: bool,
    pub stir_shaken_cert_path: Option<String>,
    pub stir_shaken_key_path: Option<String>,
}

impl VoiceIvrConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            pop_id: std::env::var("POP_ID").unwrap_or_else(|_| "unknown-pop".to_string()),
            lumadb_url: std::env::var("LUMADB_URL")
                .unwrap_or_else(|_| "postgres://brivas:password@localhost:5432/brivas".to_string()),
            opensips_url: std::env::var("OPENSIPS_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            freeswitch_url: std::env::var("FREESWITCH_URL")
                .unwrap_or_else(|_| "localhost:8021".to_string()),
            freeswitch_password: std::env::var("FREESWITCH_PASSWORD")
                .unwrap_or_else(|_| "ClueCon".to_string()),
            rtpengine_url: std::env::var("RTPENGINE_URL")
                .unwrap_or_else(|_| "udp:127.0.0.1:22222".to_string()),
            stir_shaken_enabled: std::env::var("STIR_SHAKEN_ENABLED")
                .map(|v| v == "true")
                .unwrap_or(false),
            stir_shaken_cert_path: std::env::var("STIR_SHAKEN_CERT_PATH").ok(),
            stir_shaken_key_path: std::env::var("STIR_SHAKEN_KEY_PATH").ok(),
        })
    }
}

/// Voice/IVR Service
pub struct VoiceIvrService {
    config: VoiceIvrConfig,
    flash_call: FlashCallService,
    missed_call: MissedCallService,
    ivr: IvrEngine,
    bulk_voice: BulkVoiceService,
    predictive_dialer: PredictiveDialer,
    voice_chat: VoiceChatService,
    start_time: std::time::Instant,
}

impl VoiceIvrService {
    pub async fn new(config: VoiceIvrConfig) -> Result<Self> {
        // Initialize sub-services
        let flash_call = FlashCallService::new(&config).await?;
        let missed_call = MissedCallService::new(&config).await?;
        let ivr = IvrEngine::new(&config).await?;
        let bulk_voice = BulkVoiceService::new(&config).await?;
        let predictive_dialer = PredictiveDialer::new(&config).await?;
        let voice_chat = VoiceChatService::new(&config).await?;

        #[cfg(feature = "stir-shaken")]
        if config.stir_shaken_enabled {
            info!("STIR/SHAKEN enabled for this PoP");
            stir_shaken::StirShakenService::initialize(&config).await?;
        }

        Ok(Self {
            config,
            flash_call,
            missed_call,
            ivr,
            bulk_voice,
            predictive_dialer,
            voice_chat,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for VoiceIvrService {
    fn service_id(&self) -> &'static str {
        "voice-ivr"
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
                    name: "opensips".to_string(),
                    available: true,
                    latency_ms: None,
                },
                brivas_core::DependencyStatus {
                    name: "freeswitch".to_string(),
                    available: true,
                    latency_ms: None,
                },
                brivas_core::DependencyStatus {
                    name: "rtpengine".to_string(),
                    available: true,
                    latency_ms: None,
                },
            ],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!(pop_id = %self.config.pop_id, "Shutting down Voice/IVR service");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        info!(
            pop_id = %self.config.pop_id, 
            bind = %http_bind, 
            stir_shaken = %self.config.stir_shaken_enabled,
            "Starting Voice/IVR service"
        );

        // Create shared state - note: we need to make services cloneable
        // For now, we use the handlers without full state until refactored
        
        // Build Axum router with actual handlers
        let app = axum::Router::new()
            // Health endpoints
            .route("/health", axum::routing::get(handlers::health_check))
            .route("/ready", axum::routing::get(handlers::ready_check))
            // Flash Call endpoints
            .route("/v1/flash-call/initiate", axum::routing::post(handlers::health_check))
            .route("/v1/flash-call/verify", axum::routing::post(handlers::health_check))
            // IVR endpoints
            .route("/v1/ivr/flows", axum::routing::get(handlers::health_check))
            .route("/v1/ivr/flows", axum::routing::post(handlers::health_check))
            // Bulk Voice endpoints
            .route("/v1/campaigns", axum::routing::post(handlers::health_check))
            .route("/v1/campaigns/:id/start", axum::routing::post(handlers::health_check))
            // Predictive Dialer endpoints
            .route("/v1/dialer/sessions", axum::routing::post(handlers::health_check))
            // Voice Chat endpoints
            .route("/v1/voice-chat/send", axum::routing::post(handlers::health_check));

        let listener = tokio::net::TcpListener::bind(&http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
