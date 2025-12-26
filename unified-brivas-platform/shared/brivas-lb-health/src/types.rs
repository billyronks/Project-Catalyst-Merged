//! Health monitoring types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status for a service endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointHealth {
    pub service: String,
    pub pod_ip: String,
    pub node: String,
    pub pop_id: String,
    pub healthy: bool,
    pub last_check: DateTime<Utc>,
    pub latency_ms: f64,
    pub consecutive_failures: u32,
    pub total_requests: u64,
    pub failed_requests: u64,
}

/// VIP status across the PoP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VipStatus {
    pub vip: String,
    pub service: String,
    pub pop_id: String,
    pub active_endpoints: u32,
    pub total_endpoints: u32,
    pub requests_per_second: f64,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub healthy: bool,
}

/// Cilium service from BPF maps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiliumService {
    pub id: u32,
    pub name: String,
    pub namespace: String,
    pub frontend_address: String,
    pub backend_address: String,
    pub node: String,
    pub protocol: String,
    pub port: u16,
}

/// Peer PoP status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStatus {
    pub pop_id: String,
    pub healthy: bool,
    pub latency_ms: f64,
    pub last_seen: DateTime<Utc>,
    pub reachable: bool,
}

/// Global health view across all PoPs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalHealthView {
    pub pops: HashMap<String, PopHealth>,
    pub updated_at: DateTime<Utc>,
}

/// Health status for a single PoP
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopHealth {
    pub pop_id: String,
    pub services: Vec<VipStatus>,
    pub healthy: bool,
    pub load_percentage: f64,
    pub active_connections: u64,
}

impl PopHealth {
    pub fn overall_health_score(&self) -> f64 {
        if self.services.is_empty() {
            return 0.0;
        }
        
        let healthy_count = self.services.iter().filter(|s| s.healthy).count();
        healthy_count as f64 / self.services.len() as f64
    }
    
    pub fn available_capacity_score(&self) -> f64 {
        1.0 - (self.load_percentage / 100.0).min(1.0)
    }
}

/// Latency measurement between PoPs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    pub from_pop: String,
    pub to_pop: String,
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub samples: u32,
    pub measured_at: DateTime<Utc>,
}

/// Health check errors
#[derive(Debug, thiserror::Error)]
pub enum HealthError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Cilium CLI error: {0}")]
    CiliumCli(String),
}

/// Coordinator errors
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    #[error("No healthy PoP available")]
    NoHealthyPop,
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Unknown PoP: {0}")]
    UnknownPop(String),
}
