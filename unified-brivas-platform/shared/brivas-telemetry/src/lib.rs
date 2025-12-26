//! Brivas Telemetry
//!
//! Unified observability: tracing, metrics, and distributed tracing via OpenTelemetry.

mod config;
mod tracing_setup;
mod metrics;

pub use config::TelemetryConfig;
pub use tracing_setup::init_tracing;
pub use metrics::{Counter, Histogram, Gauge};

/// Initialize all telemetry for a service
pub fn init(service_name: &str) -> Result<TelemetryGuard, TelemetryError> {
    let config = TelemetryConfig::from_env();
    init_tracing(service_name, &config)?;
    Ok(TelemetryGuard { _private: () })
}

/// Guard that shuts down telemetry on drop
pub struct TelemetryGuard {
    _private: (),
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Tracing initialization failed: {0}")]
    TracingInit(String),

    #[error("OTLP configuration error: {0}")]
    OtlpConfig(String),
}
