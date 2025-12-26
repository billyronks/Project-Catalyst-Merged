//! Billing Service
//!
//! Carrier-grade billing system for VAS platform:
//! - CDR (Call Detail Record) collection and processing
//! - Real-time rating engine with LCR support
//! - Invoice generation and management
//! - Prepaid wallet with real-time balance updates
//! - Usage analytics and reporting

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod cdr;
mod rating;
mod invoice;
mod wallet;
mod api;
mod types;

#[cfg(test)]
mod tests;

pub use cdr::CdrCollector;
pub use rating::RatingEngine;
pub use invoice::InvoiceService;
pub use wallet::WalletService;
pub use types::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("billing=debug".parse().expect("valid tracing directive")),
        )
        .json()
        .init();

    info!("Starting Billing Service");

    let service = Arc::new(BillingService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct BillingService {
    config: BillingConfig,
    cdr_collector: CdrCollector,
    rating_engine: RatingEngine,
    invoice_service: InvoiceService,
    wallet_service: WalletService,
    start_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct BillingConfig {
    pub http_bind: String,
    pub grpc_bind: String,
    pub lumadb_url: String,
    pub default_currency: String,
    pub invoice_day: u8,
    pub prepaid_low_balance_threshold: rust_decimal::Decimal,
}

impl BillingConfig {
    pub fn from_env() -> Result<Self> {
        use rust_decimal_macros::dec;
        Ok(Self {
            http_bind: std::env::var("HTTP_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            grpc_bind: std::env::var("GRPC_BIND")
                .unwrap_or_else(|_| "0.0.0.0:9090".to_string()),
            lumadb_url: std::env::var("LUMADB_URL")
                .unwrap_or_else(|_| "postgres://brivas:password@localhost:5432/brivas".to_string()),
            default_currency: std::env::var("DEFAULT_CURRENCY")
                .unwrap_or_else(|_| "NGN".to_string()),
            invoice_day: std::env::var("INVOICE_DAY")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            prepaid_low_balance_threshold: std::env::var("LOW_BALANCE_THRESHOLD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(dec!(1000.00)),
        })
    }
}

impl BillingService {
    pub async fn new() -> Result<Self> {
        let config = BillingConfig::from_env()?;
        
        let cdr_collector = CdrCollector::new(&config.lumadb_url).await?;
        let rating_engine = RatingEngine::new(&config.lumadb_url).await?;
        let invoice_service = InvoiceService::new(&config.lumadb_url, &config.default_currency).await?;
        let wallet_service = WalletService::new(&config.lumadb_url, config.prepaid_low_balance_threshold).await?;

        Ok(Self {
            config,
            cdr_collector,
            rating_engine,
            invoice_service,
            wallet_service,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for BillingService {
    fn service_id(&self) -> &'static str {
        "billing"
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
        info!("Shutting down Billing Service");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            grpc = %self.config.grpc_bind,
            "Starting Billing servers"
        );

        // Create REST router
        let rest_router = api::rest::create_router(
            self.cdr_collector.clone(),
            self.rating_engine.clone(),
            self.invoice_service.clone(),
            self.wallet_service.clone(),
        );

        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, rest_router).await?;

        Ok(())
    }
}
