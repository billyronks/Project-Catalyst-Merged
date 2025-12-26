//! Load Balancer Health Monitor
//!
//! Continuously monitors Cilium eBPF load balancer health and stores
//! metrics in LumaDB for global visibility.

use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::Utc;
use dashmap::DashMap;
use tokio::time::interval;

use crate::types::*;

/// Load balancer health monitor
pub struct LbHealthMonitor {
    pop_id: String,
    check_interval: Duration,
    // Placeholder for LumaDB client
    #[allow(dead_code)]
    db_url: String,
    endpoints: Arc<DashMap<String, EndpointHealth>>,
    vips: Arc<DashMap<String, VipStatus>>,
}

impl LbHealthMonitor {
    pub fn new(pop_id: String, db_url: String) -> Self {
        Self {
            pop_id,
            db_url,
            check_interval: Duration::from_secs(5),
            endpoints: Arc::new(DashMap::new()),
            vips: Arc::new(DashMap::new()),
        }
    }
    
    /// Start continuous health monitoring
    pub async fn start(&self) -> Result<(), HealthError> {
        let mut ticker = interval(self.check_interval);
        
        loop {
            ticker.tick().await;
            
            // Get all services from Cilium
            let services = self.get_cilium_services().await?;
            
            for service in services {
                let health = self.check_service_health(&service).await?;
                
                // Store endpoint health
                let key = format!("{}:{}", self.pop_id, health.pod_ip);
                self.endpoints.insert(key, health.clone());
                
                // Calculate and store VIP status
                let vip_status = self.calculate_vip_status(&service).await?;
                let vip_key = format!("{}:{}", self.pop_id, vip_status.vip);
                self.vips.insert(vip_key, vip_status);
            }
            
            tracing::debug!(
                pop_id = %self.pop_id,
                endpoints = self.endpoints.len(),
                vips = self.vips.len(),
                "Health check cycle complete"
            );
        }
    }
    
    /// Get services from Cilium eBPF maps via CLI
    async fn get_cilium_services(&self) -> Result<Vec<CiliumService>, HealthError> {
        let output = tokio::process::Command::new("cilium")
            .args(["service", "list", "-o", "json"])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(HealthError::CiliumCli(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        let services: Vec<CiliumService> = serde_json::from_slice(&output.stdout)
            .map_err(|e| HealthError::Parse(e.to_string()))?;
        
        Ok(services)
    }
    
    /// Perform health check on service endpoint
    async fn check_service_health(&self, service: &CiliumService) -> Result<EndpointHealth, HealthError> {
        let start = Instant::now();
        
        // TCP connect check
        let healthy = tokio::net::TcpStream::connect(&service.backend_address)
            .await
            .is_ok();
        
        let latency = start.elapsed();
        
        // Get existing state for consecutive failures
        let prev_failures = self.endpoints
            .get(&format!("{}:{}", self.pop_id, service.backend_address))
            .map(|e| e.consecutive_failures)
            .unwrap_or(0);
        
        Ok(EndpointHealth {
            service: service.name.clone(),
            pod_ip: service.backend_address.clone(),
            node: service.node.clone(),
            pop_id: self.pop_id.clone(),
            healthy,
            last_check: Utc::now(),
            latency_ms: latency.as_secs_f64() * 1000.0,
            consecutive_failures: if healthy { 0 } else { prev_failures + 1 },
            total_requests: 1,
            failed_requests: if healthy { 0 } else { 1 },
        })
    }
    
    /// Calculate VIP status from backend endpoints
    async fn calculate_vip_status(&self, service: &CiliumService) -> Result<VipStatus, HealthError> {
        // Count healthy backends for this service
        let (active, total) = self.endpoints
            .iter()
            .filter(|e| e.service == service.name)
            .fold((0u32, 0u32), |(active, total), e| {
                (active + if e.healthy { 1 } else { 0 }, total + 1)
            });
        
        // Calculate average latency
        let avg_latency = self.endpoints
            .iter()
            .filter(|e| e.service == service.name && e.healthy)
            .map(|e| e.latency_ms)
            .sum::<f64>() / active.max(1) as f64;
        
        Ok(VipStatus {
            vip: service.frontend_address.clone(),
            service: service.name.clone(),
            pop_id: self.pop_id.clone(),
            active_endpoints: active,
            total_endpoints: total,
            requests_per_second: 0.0, // Would be from actual metrics
            avg_latency_ms: avg_latency,
            p99_latency_ms: avg_latency * 1.5, // Estimate
            healthy: active > 0,
        })
    }
    
    /// Get current health snapshot
    pub fn get_health_snapshot(&self) -> Vec<EndpointHealth> {
        self.endpoints.iter().map(|e| e.value().clone()).collect()
    }
    
    /// Get VIP status snapshot
    pub fn get_vip_snapshot(&self) -> Vec<VipStatus> {
        self.vips.iter().map(|v| v.value().clone()).collect()
    }
}
