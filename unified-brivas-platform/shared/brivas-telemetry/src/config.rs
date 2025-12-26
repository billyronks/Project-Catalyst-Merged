//! Telemetry Configuration

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub otlp_endpoint: Option<String>,
    pub log_level: String,
    pub json_logs: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "brivas-service".to_string(),
            otlp_endpoint: None,
            log_level: "info".to_string(),
            json_logs: true,
        }
    }
}

impl TelemetryConfig {
    pub fn from_env() -> Self {
        Self {
            service_name: std::env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "brivas-service".to_string()),
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            log_level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            json_logs: std::env::var("JSON_LOGS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        }
    }
}
