//! API Handlers for Voice/IVR endpoints
//!
//! Simplified handlers with uniform response types

use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

// =============================================================================
// Health Handlers
// =============================================================================

/// Health check
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "healthy"
    })))
}

/// Readiness check
pub async fn ready_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ready"
    })))
}
