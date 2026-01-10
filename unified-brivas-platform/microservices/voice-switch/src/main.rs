//! Voice Switch Microservice
//!
//! High-performance Class 4/5 VoIP softswitch with:
//! - Carrier management and failover
//! - Least Cost Routing (LCR) engine
//! - QuestDB real-time analytics (11.4M rows/sec, sub-2ms latency)
//! - ML-powered fraud detection (IRSF, Wangiri, velocity)
//! - Circuit breaker pattern for carrier failover
//! - WebRTC session management
//! - STIR/SHAKEN verification

mod analytics;
mod carrier;
mod circuit_breaker;
mod config;
mod error;
mod fraud;
mod handlers;
mod kdb;
mod lcr;
mod routes;
mod webrtc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub use config::Config;
pub use error::{Error, Result};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: brivas_lumadb::LumaDbPool,
    pub kdb_client: Arc<kdb::KdbClient>,
    pub carrier_cache: Arc<carrier::CarrierCache>,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .json()
        .init();

    info!("Starting Voice Switch microservice");

    // Load configuration
    let config = Config::from_env()?;
    let bind_addr = config.bind_address();

    // Initialize database pool
    let db = brivas_lumadb::LumaDbPool::new(&config.database_url).await?;

    // Initialize kdb+ client
    let kdb_client = Arc::new(kdb::KdbClient::new(&config.kdb_host, config.kdb_port).await?);

    // Initialize carrier cache
    let carrier_cache = Arc::new(carrier::CarrierCache::new());

    // Build application state
    let state = AppState {
        db,
        kdb_client,
        carrier_cache,
        config: Arc::new(config),
    };

    // Build router
    let app = routes::create_router(state);

    // Start server
    let listener = TcpListener::bind(&bind_addr).await?;
    info!("Voice Switch listening on {}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
