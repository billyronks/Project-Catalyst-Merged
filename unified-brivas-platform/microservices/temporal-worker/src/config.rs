//! Configuration for Temporal Worker

use std::net::SocketAddr;

/// Temporal worker configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP health server host
    pub host: String,
    /// HTTP health server port
    pub port: u16,
    /// Temporal server host
    pub temporal_host: String,
    /// Temporal server port
    pub temporal_port: u16,
    /// Temporal namespace
    pub temporal_namespace: String,
    /// Task queue name
    pub task_queue: String,
    /// LumaDB connection URL
    pub database_url: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8096".to_string())
                .parse()?,
            temporal_host: std::env::var("TEMPORAL_HOST")
                .unwrap_or_else(|_| "temporal".to_string()),
            temporal_port: std::env::var("TEMPORAL_PORT")
                .unwrap_or_else(|_| "7233".to_string())
                .parse()?,
            temporal_namespace: std::env::var("TEMPORAL_NAMESPACE")
                .unwrap_or_else(|_| "brivas".to_string()),
            task_queue: std::env::var("TEMPORAL_TASK_QUEUE")
                .unwrap_or_else(|_| "brivas-workflows".to_string()),
            database_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:brivas_secret@lumadb:5432/brivas".to_string()
            }),
        })
    }

    /// Get socket address for binding health server
    pub fn bind_address(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid bind address")
    }
}
