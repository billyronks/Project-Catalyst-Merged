//! Error types for Temporal SDK

/// Result type alias
pub type Result<T> = std::result::Result<T, TemporalError>;

/// Temporal SDK errors
#[derive(Debug, thiserror::Error)]
pub enum TemporalError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Activity failed: {0}")]
    ActivityFailed(String),

    #[error("Workflow timeout: {0}")]
    Timeout(String),

    #[error("Workflow cancelled: {0}")]
    Cancelled(String),

    #[error("Invalid workflow state: {0}")]
    InvalidState(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for TemporalError {
    fn from(err: serde_json::Error) -> Self {
        TemporalError::Serialization(err.to_string())
    }
}
