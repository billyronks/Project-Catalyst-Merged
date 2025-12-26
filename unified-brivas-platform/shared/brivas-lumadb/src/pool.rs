//! Connection Pool for LumaDB

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;
use tracing::{debug, info};

use crate::{LumaDbError, Result};

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub url: String,
    pub max_size: usize,
    pub min_idle: Option<usize>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            url: "postgres://brivas:password@localhost:5432/brivas".to_string(),
            max_size: 32,
            min_idle: Some(4),
        }
    }
}

impl PoolConfig {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("LUMADB_URL")
                .unwrap_or_else(|_| "postgres://brivas:password@localhost:5432/brivas".to_string()),
            max_size: std::env::var("LUMADB_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(32),
            min_idle: std::env::var("LUMADB_MIN_IDLE")
                .ok()
                .and_then(|s| s.parse().ok()),
        }
    }
}

/// LumaDB Connection Pool
#[derive(Clone)]
pub struct LumaDbPool {
    pool: Pool,
}

impl LumaDbPool {
    /// Create a new connection pool
    pub async fn new(config: PoolConfig) -> Result<Self> {
        info!(
            max_size = config.max_size,
            "Creating LumaDB connection pool"
        );

        let pg_config: tokio_postgres::Config = config.url.parse()
            .map_err(|e| LumaDbError::Configuration(format!("Invalid URL: {}", e)))?;

        let manager_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };

        let manager = Manager::from_config(pg_config, NoTls, manager_config);

        let pool = Pool::builder(manager)
            .max_size(config.max_size)
            .build()
            .map_err(|e| LumaDbError::Pool(e.to_string()))?;

        debug!("LumaDB pool created successfully");

        Ok(Self { pool })
    }

    /// Get a connection from the pool
    pub async fn get(&self) -> Result<deadpool_postgres::Object> {
        self.pool
            .get()
            .await
            .map_err(|e| LumaDbError::Pool(e.to_string()))
    }

    /// Check pool health
    pub async fn is_healthy(&self) -> bool {
        match self.pool.get().await {
            Ok(conn) => conn.simple_query("SELECT 1").await.is_ok(),
            Err(_) => false,
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let status = self.pool.status();
        PoolStats {
            size: status.size,
            available: status.available as usize,
            waiting: status.waiting,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: usize,
    pub available: usize,
    pub waiting: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_size, 32);
        assert_eq!(config.min_idle, Some(4));
    }
}
