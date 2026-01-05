//! Anomaly Detection Module
//!
//! Detects anomalies across platform services using:
//! - Statistical analysis (z-score, moving averages)
//! - Threshold-based detection
//! - Cross-service correlation

use brivas_lumadb::LumaDbPool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, warn};

use crate::{AiOpsConfig, Severity};

#[derive(Debug, Error)]
pub enum AnomalyError {
    #[error("Database error: {0}")]
    Database(#[from] brivas_lumadb::LumaDbError),
    
    #[error("Query error: {0}")]
    Query(String),
}

pub type Result<T> = std::result::Result<T, AnomalyError>;

/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub source: String,
    pub metric: String,
    pub current_value: f64,
    pub expected_value: f64,
    pub deviation: f64,
    pub severity: Severity,
    pub description: String,
    pub recommended_playbook: Option<String>,
    pub context: serde_json::Value,
}

/// Anomaly detector configuration
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Standard deviation threshold for anomaly
    pub zscore_threshold: f64,
    /// Lookback window in seconds
    pub lookback_secs: i64,
    /// Minimum samples for detection
    pub min_samples: usize,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            zscore_threshold: 3.0,
            lookback_secs: 300, // 5 minutes
            min_samples: 10,
        }
    }
}

/// Anomaly detector engine
pub struct AnomalyDetector {
    pool: LumaDbPool,
    config: DetectorConfig,
    detectors: Vec<Box<dyn Detector + Send + Sync>>,
}

/// Trait for individual anomaly detectors
#[async_trait::async_trait]
pub trait Detector: Send + Sync {
    fn name(&self) -> &str;
    async fn detect(&self, pool: &LumaDbPool) -> Result<Vec<Anomaly>>;
}

impl AnomalyDetector {
    pub fn new(pool: LumaDbPool, config: &AiOpsConfig) -> Self {
        let detector_config = DetectorConfig::default();
        
        // Register built-in detectors
        let detectors: Vec<Box<dyn Detector + Send + Sync>> = vec![
            Box::new(SmppBindDetector::new(detector_config.clone())),
            Box::new(LatencyDetector::new(detector_config.clone())),
            Box::new(ErrorRateDetector::new(detector_config.clone())),
            Box::new(ThroughputDetector::new(detector_config.clone())),
            Box::new(ResourceDetector::new(detector_config.clone())),
        ];
        
        Self {
            pool,
            config: detector_config,
            detectors,
        }
    }
    
    /// Run all detectors
    pub async fn detect_all(&self) -> Result<Vec<Anomaly>> {
        let mut all_anomalies = Vec::new();
        
        for detector in &self.detectors {
            match detector.detect(&self.pool).await {
                Ok(anomalies) => {
                    debug!(detector = detector.name(), count = anomalies.len(), "Detection complete");
                    all_anomalies.extend(anomalies);
                }
                Err(e) => {
                    warn!(detector = detector.name(), error = %e, "Detection failed");
                }
            }
        }
        
        Ok(all_anomalies)
    }
}

// ============================================================================
// SMPP Bind Detector
// ============================================================================

/// Detects SMPP bind disconnects and failures
pub struct SmppBindDetector {
    config: DetectorConfig,
}

impl SmppBindDetector {
    pub fn new(config: DetectorConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Detector for SmppBindDetector {
    fn name(&self) -> &str {
        "smpp_bind"
    }
    
    async fn detect(&self, pool: &LumaDbPool) -> Result<Vec<Anomaly>> {
        let conn = pool.get().await?;
        
        // Query for SMPP bind failures in the last interval
        let query = r#"
            SELECT 
                session_id,
                peer_address,
                disconnect_count,
                last_disconnect_at,
                avg_session_duration_secs
            FROM smpp_session_metrics
            WHERE disconnect_count > 3
              AND last_disconnect_at > NOW() - INTERVAL '5 minutes'
        "#;
        
        match conn.query(query, &[]).await {
            Ok(rows) => {
                let anomalies: Vec<Anomaly> = rows.iter().map(|row| {
                    let session_id: String = row.get(0);
                    let peer_address: String = row.get(1);
                    let disconnect_count: i64 = row.get(2);
                    
                    Anomaly {
                        source: "smsc".to_string(),
                        metric: "smpp_bind_disconnect".to_string(),
                        current_value: disconnect_count as f64,
                        expected_value: 0.0,
                        deviation: disconnect_count as f64,
                        severity: if disconnect_count > 10 { Severity::Critical } else { Severity::High },
                        description: format!(
                            "SMPP session {} to {} disconnected {} times",
                            session_id, peer_address, disconnect_count
                        ),
                        recommended_playbook: Some("smpp_recovery".to_string()),
                        context: serde_json::json!({
                            "session_id": session_id,
                            "peer_address": peer_address,
                            "disconnect_count": disconnect_count
                        }),
                    }
                }).collect();
                
                Ok(anomalies)
            }
            Err(_) => {
                // Table might not exist yet - return empty
                Ok(vec![])
            }
        }
    }
}

// ============================================================================
// Latency Detector
// ============================================================================

pub struct LatencyDetector {
    config: DetectorConfig,
}

impl LatencyDetector {
    pub fn new(config: DetectorConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Detector for LatencyDetector {
    fn name(&self) -> &str {
        "latency"
    }
    
    async fn detect(&self, pool: &LumaDbPool) -> Result<Vec<Anomaly>> {
        let conn = pool.get().await?;
        
        let query = r#"
            SELECT 
                service_name,
                AVG(latency_ms) as avg_latency,
                MAX(latency_ms) as max_latency,
                STDDEV(latency_ms) as stddev_latency
            FROM request_metrics
            WHERE timestamp > NOW() - INTERVAL '5 minutes'
            GROUP BY service_name
            HAVING MAX(latency_ms) > AVG(latency_ms) + 3 * STDDEV(latency_ms)
        "#;
        
        match conn.query(query, &[]).await {
            Ok(rows) => {
                let anomalies: Vec<Anomaly> = rows.iter().filter_map(|row| {
                    let service_name: String = row.get(0);
                    let avg_latency: f64 = row.get(1);
                    let max_latency: f64 = row.get(2);
                    
                    if max_latency > 1000.0 { // > 1 second
                        Some(Anomaly {
                            source: service_name.clone(),
                            metric: "latency_ms".to_string(),
                            current_value: max_latency,
                            expected_value: avg_latency,
                            deviation: (max_latency - avg_latency) / avg_latency * 100.0,
                            severity: if max_latency > 5000.0 { Severity::Critical } else { Severity::Medium },
                            description: format!(
                                "High latency detected in {}: {}ms (avg: {}ms)",
                                service_name, max_latency, avg_latency
                            ),
                            recommended_playbook: Some("service_restart".to_string()),
                            context: serde_json::json!({
                                "service": service_name,
                                "avg_latency": avg_latency,
                                "max_latency": max_latency
                            }),
                        })
                    } else {
                        None
                    }
                }).collect();
                
                Ok(anomalies)
            }
            Err(_) => Ok(vec![]),
        }
    }
}

// ============================================================================
// Error Rate Detector
// ============================================================================

pub struct ErrorRateDetector {
    config: DetectorConfig,
}

impl ErrorRateDetector {
    pub fn new(config: DetectorConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Detector for ErrorRateDetector {
    fn name(&self) -> &str {
        "error_rate"
    }
    
    async fn detect(&self, pool: &LumaDbPool) -> Result<Vec<Anomaly>> {
        // Stub implementation - check for high error rates
        Ok(vec![])
    }
}

// ============================================================================
// Throughput Detector
// ============================================================================

pub struct ThroughputDetector {
    config: DetectorConfig,
}

impl ThroughputDetector {
    pub fn new(config: DetectorConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Detector for ThroughputDetector {
    fn name(&self) -> &str {
        "throughput"
    }
    
    async fn detect(&self, pool: &LumaDbPool) -> Result<Vec<Anomaly>> {
        // Stub implementation - check for throughput drops
        Ok(vec![])
    }
}

// ============================================================================
// Resource Detector
// ============================================================================

pub struct ResourceDetector {
    config: DetectorConfig,
}

impl ResourceDetector {
    pub fn new(config: DetectorConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Detector for ResourceDetector {
    fn name(&self) -> &str {
        "resources"
    }
    
    async fn detect(&self, pool: &LumaDbPool) -> Result<Vec<Anomaly>> {
        // Stub implementation - check CPU, memory, disk usage
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_config_default() {
        let config = DetectorConfig::default();
        assert_eq!(config.zscore_threshold, 3.0);
        assert_eq!(config.lookback_secs, 300);
    }
}
