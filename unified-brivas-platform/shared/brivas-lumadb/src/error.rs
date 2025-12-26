//! LumaDB Error Types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LumaDbError>;

#[derive(Debug, Error)]
pub enum LumaDbError {
    #[error("Connection error: {0}")]
    Connection(#[from] tokio_postgres::Error),

    #[error("Query error: {0}")]
    Query(tokio_postgres::Error),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Row not found")]
    NotFound,
}
