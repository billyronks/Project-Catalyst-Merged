//! Tracing Setup

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{TelemetryConfig, TelemetryError};

/// Initialize tracing with OpenTelemetry
pub fn init_tracing(service_name: &str, config: &TelemetryConfig) -> Result<(), TelemetryError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    // Build subscriber based on JSON logging preference
    if config.json_logs {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true);
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| TelemetryError::TracingInit(e.to_string()))?;
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true);
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| TelemetryError::TracingInit(e.to_string()))?;
    }

    tracing::info!(
        service = service_name,
        log_level = %config.log_level,
        json_logs = config.json_logs,
        "Tracing initialized"
    );

    Ok(())
}
