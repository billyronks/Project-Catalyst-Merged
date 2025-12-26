//! Error types for Brivas services

use thiserror::Error;

pub type Result<T> = std::result::Result<T, BrivasError>;

#[derive(Error, Debug)]
pub enum BrivasError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Authorization error: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Service unavailable: {0}")]
    Unavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

impl BrivasError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Validation(_) => 400,
            Self::Auth(_) => 401,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::RateLimited(_) => 429,
            Self::Unavailable(_) => 503,
            Self::Timeout(_) => 504,
            _ => 500,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Config(_) => "CONFIG_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Network(_) => "NETWORK_ERROR",
            Self::Auth(_) => "AUTH_ERROR",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::RateLimited(_) => "RATE_LIMITED",
            Self::Unavailable(_) => "UNAVAILABLE",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Protocol(_) => "PROTOCOL_ERROR",
            Self::Timeout(_) => "TIMEOUT",
        }
    }
}

impl From<std::io::Error> for BrivasError {
    fn from(err: std::io::Error) -> Self {
        BrivasError::Network(err.to_string())
    }
}
