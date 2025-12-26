//! Configuration management for microservices

use crate::error::{BrivasError, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub http_port: u16,
    pub grpc_port: u16,
    pub lumadb_url: String,
    pub log_level: String,
    pub enable_telemetry: bool,
}

impl ServiceConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "unknown".to_string()),
            http_port: env::var("HTTP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e| BrivasError::Config(format!("Invalid HTTP_PORT: {}", e)))?,
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()
                .map_err(|e| BrivasError::Config(format!("Invalid GRPC_PORT: {}", e)))?,
            lumadb_url: env::var("LUMADB_URL")
                .unwrap_or_else(|_| "postgres://brivas:password@localhost:5432/brivas".to_string()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            enable_telemetry: env::var("ENABLE_TELEMETRY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}
