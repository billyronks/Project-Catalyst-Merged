//! Error types for kdb+ SDK

/// Result type alias
pub type Result<T> = std::result::Result<T, KdbError>;

/// kdb+ client errors
#[derive(Debug, thiserror::Error)]
pub enum KdbError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Not connected to kdb+ server")]
    NotConnected,

    #[error("IO error: {0}")]
    IO(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Type conversion error: {0}")]
    TypeConversion(String),
}
