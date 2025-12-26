//! STIR/SHAKEN Authentication Service
//!
//! Centralized caller ID authentication service providing:
//! - PASSporT signing for outbound calls (A/B/C attestation)
//! - PASSporT verification for inbound calls
//! - Certificate lifecycle management
//! - STI-PA/STI-CA integration
//! - CRL/OCSP checking

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod config;
mod api;
mod certificate;
mod attestation;
mod verification;
mod acme;
mod infrastructure;
mod types;

pub use config::StirShakenConfig;
pub use types::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("stir_shaken_service=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting STIR/SHAKEN Authentication Service");

    let service = Arc::new(StirShakenService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct StirShakenService {
    config: StirShakenConfig,
    cert_manager: certificate::CertificateManager,
    signer: attestation::AttestationSigner,
    verifier: verification::VerificationService,
    start_time: std::time::Instant,
}

impl StirShakenService {
    pub async fn new() -> Result<Self> {
        let config = StirShakenConfig::from_env()?;
        let cert_manager = certificate::CertificateManager::new(&config.lumadb_url).await?;
        let signer = attestation::AttestationSigner::new(cert_manager.clone());
        let verifier = verification::VerificationService::new(&config).await?;

        Ok(Self {
            config,
            cert_manager,
            signer,
            verifier,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for StirShakenService {
    fn service_id(&self) -> &'static str {
        "stir-shaken-service"
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
                    name: "certificates".to_string(),
                    available: self.cert_manager.has_active_certificate(),
                    latency_ms: None,
                },
            ],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down STIR/SHAKEN service");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            "Starting STIR/SHAKEN HTTP server"
        );

        // Start REST management API
        let rest_router = api::rest::create_router(&self.cert_manager);
        let http_listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(http_listener, rest_router).await?;

        Ok(())
    }
}
