//! Common types shared across services

use serde::{Deserialize, Serialize};

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub request_id: String,
}

/// API error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Pagination request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaginationRequest {
    pub page: u32,
    pub page_size: u32,
    pub cursor: Option<String>,
}

/// Pagination response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponse {
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service_id: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Timestamp wrapper (milliseconds since epoch)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp_millis())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt.timestamp_millis())
    }
}
