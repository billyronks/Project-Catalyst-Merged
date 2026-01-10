//! Configuration for Voice Switch microservice

use std::net::SocketAddr;

/// Voice Switch configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP bind address
    pub host: String,
    /// HTTP port
    pub port: u16,
    /// LumaDB connection URL
    pub database_url: String,
    /// kdb+ host
    pub kdb_host: String,
    /// kdb+ port
    pub kdb_port: u16,
    /// Cache TTL for carriers (seconds)
    pub carrier_cache_ttl_secs: u64,
    /// Cache TTL for routes (seconds)
    pub route_cache_ttl_secs: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8095".to_string())
                .parse()?,
            database_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:brivas_secret@lumadb:5432/brivas".to_string()
            }),
            kdb_host: std::env::var("KDB_HOST").unwrap_or_else(|_| "kdb-gateway".to_string()),
            kdb_port: std::env::var("KDB_PORT")
                .unwrap_or_else(|_| "5010".to_string())
                .parse()?,
            carrier_cache_ttl_secs: std::env::var("CARRIER_CACHE_TTL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()?,
            route_cache_ttl_secs: std::env::var("ROUTE_CACHE_TTL")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
        })
    }

    /// Get socket address for binding
    pub fn bind_address(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid bind address")
    }
}
