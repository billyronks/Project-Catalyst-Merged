//! Error types for Voice Switch

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Voice Switch error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Carrier not found: {0}")]
    CarrierNotFound(String),

    #[error("Route not found: {0}")]
    RouteNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    #[error("kdb+ error: {0}")]
    Kdb(String),

    #[error("No route available for destination: {0}")]
    NoRouteAvailable(String),

    #[error("Carrier unavailable: {0}")]
    CarrierUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Error::CarrierNotFound(_) | Error::RouteNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            Error::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::NoRouteAvailable(_) | Error::CarrierUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            Error::Database(_) | Error::Kdb(_) | Error::Internal(_) => {
                tracing::error!("Internal error: {:?}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}
