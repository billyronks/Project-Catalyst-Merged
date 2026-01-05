//! Metrics Collection Module
//!
//! Collects and aggregates metrics for anomaly detection

use brivas_lumadb::LumaDbPool;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Database error: {0}")]
    Database(#[from] brivas_lumadb::LumaDbError),
    
    #[error("Collection error: {0}")]
    Collection(String),
}

pub type Result<T> = std::result::Result<T, MetricsError>;

/// Metrics collector
pub struct MetricsCollector {
    pool: LumaDbPool,
}

impl MetricsCollector {
    pub fn new(pool: LumaDbPool) -> Self {
        Self { pool }
    }
    
    /// Collect service health metrics
    pub async fn collect_service_health(&self) -> Result<HashMap<String, ServiceHealth>> {
        let conn = self.pool.get().await?;
        
        // Query service health metrics
        let query = r#"
            SELECT 
                service_name,
                status,
                uptime_seconds,
                last_heartbeat
            FROM service_health
            WHERE last_heartbeat > NOW() - INTERVAL '5 minutes'
        "#;
        
        let mut health_map = HashMap::new();
        
        if let Ok(rows) = conn.query(query, &[]).await {
            for row in rows {
                let name: String = row.get(0);
                let status: String = row.get(1);
                let uptime: i64 = row.get(2);
                
                health_map.insert(name.clone(), ServiceHealth {
                    name,
                    status,
                    uptime_seconds: uptime as u64,
                    healthy: true,
                });
            }
        }
        
        Ok(health_map)
    }
    
    /// Collect SMPP session metrics
    pub async fn collect_smpp_metrics(&self) -> Result<Vec<SmppSessionMetrics>> {
        let conn = self.pool.get().await?;
        
        let query = r#"
            SELECT 
                session_id,
                peer_address,
                state,
                messages_sent,
                messages_received,
                last_activity
            FROM smpp_sessions
            WHERE state != 'closed'
        "#;
        
        let mut metrics = Vec::new();
        
        if let Ok(rows) = conn.query(query, &[]).await {
            for row in rows {
                metrics.push(SmppSessionMetrics {
                    session_id: row.get(0),
                    peer_address: row.get(1),
                    state: row.get(2),
                    messages_sent: row.get(3),
                    messages_received: row.get(4),
                });
            }
        }
        
        Ok(metrics)
    }
}

#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub name: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub healthy: bool,
}

#[derive(Debug, Clone)]
pub struct SmppSessionMetrics {
    pub session_id: String,
    pub peer_address: String,
    pub state: String,
    pub messages_sent: i64,
    pub messages_received: i64,
}
