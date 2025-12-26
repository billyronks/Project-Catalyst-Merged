//! MCP Transport layer

use serde::{Deserialize, Serialize};

/// Transport type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Stdio,
    Sse,
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(rename = "type")]
    pub transport_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl TransportConfig {
    pub fn stdio() -> Self {
        Self {
            transport_type: "stdio".to_string(),
            url: None,
        }
    }

    pub fn sse(url: impl Into<String>) -> Self {
        Self {
            transport_type: "sse".to_string(),
            url: Some(url.into()),
        }
    }
}
