//! Common Types for LumaDB

use serde::{Deserialize, Serialize};

/// Stream message for LumaDB Streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub id: String,
    pub stream: String,
    pub data: serde_json::Value,
    pub timestamp: i64,
}

/// Key-Value entry for LumaDB KV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub ttl_seconds: Option<i64>,
}

/// Time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub metric: String,
    pub value: f64,
    pub timestamp: i64,
    pub tags: std::collections::HashMap<String, String>,
}
