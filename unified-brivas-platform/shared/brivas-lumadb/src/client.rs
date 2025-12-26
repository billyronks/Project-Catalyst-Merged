//! LumaDB Client

use tokio_postgres::{Client, NoTls, Row};
use tracing::{debug, instrument};

use crate::{LumaDbError, Result};

/// LumaDB Client
/// 
/// Wraps tokio-postgres for LumaDB connections via PostgreSQL wire protocol.
#[derive(Debug)]
pub struct LumaDbClient {
    client: Client,
    #[allow(dead_code)]
    connection_url: String,
}

impl LumaDbClient {
    /// Connect to LumaDB
    #[instrument(skip(url))]
    pub async fn connect(url: &str) -> Result<Self> {
        debug!("Connecting to LumaDB");
        
        let (client, connection) = tokio_postgres::connect(url, NoTls)
            .await
            .map_err(LumaDbError::Connection)?;

        // Spawn connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("LumaDB connection error: {}", e);
            }
        });

        Ok(Self {
            client,
            connection_url: url.to_string(),
        })
    }

    /// Execute a query and return rows
    #[instrument(skip(self, params))]
    pub async fn query<T>(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<Row>>
    where
        T: for<'a> serde::Deserialize<'a>,
    {
        self.client
            .query(sql, params)
            .await
            .map_err(LumaDbError::Query)
    }

    /// Execute a query and return a single row
    pub async fn query_one(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Row> {
        self.client
            .query_one(sql, params)
            .await
            .map_err(LumaDbError::Query)
    }

    /// Execute a query and return optional row
    pub async fn query_opt(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Option<Row>> {
        self.client
            .query_opt(sql, params)
            .await
            .map_err(LumaDbError::Query)
    }

    /// Execute a statement (INSERT, UPDATE, DELETE)
    #[instrument(skip(self, params))]
    pub async fn execute(&self, sql: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64> {
        self.client
            .execute(sql, params)
            .await
            .map_err(LumaDbError::Query)
    }

    /// Execute batch statements
    pub async fn batch_execute(&self, sql: &str) -> Result<()> {
        self.client
            .batch_execute(sql)
            .await
            .map_err(LumaDbError::Query)
    }

    /// Check if connection is healthy
    pub async fn is_healthy(&self) -> bool {
        self.client
            .simple_query("SELECT 1")
            .await
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        // This test requires a running LumaDB instance
        // Skip in CI without database
        if std::env::var("LUMADB_URL").is_err() {
            return;
        }

        let url = std::env::var("LUMADB_URL").unwrap();
        let client = LumaDbClient::connect(&url).await;
        assert!(client.is_ok());
    }
}
