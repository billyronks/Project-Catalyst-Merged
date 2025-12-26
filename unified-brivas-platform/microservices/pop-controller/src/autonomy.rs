//! Autonomy Controller
//!
//! Manages PoP operation modes during network partitions.

use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use dashmap::DashMap;
use tokio::sync::RwLock;
use tokio::time::interval;

/// Autonomous operation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomyMode {
    /// Normal operation - full connectivity to other PoPs
    Connected,
    /// Partial connectivity - some PoPs unreachable
    Degraded,
    /// Complete isolation - operating fully independently
    Isolated,
}

/// Peer status
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub pop_id: String,
    pub healthy: bool,
    pub last_seen: chrono::DateTime<Utc>,
    pub latency_ms: Option<f64>,
}

/// PoP Autonomy Controller
#[derive(Clone)]
pub struct AutonomyController {
    pop_id: String,
    #[allow(dead_code)]
    db_url: String,
    mode: Arc<RwLock<AutonomyMode>>,
    peer_status: Arc<DashMap<String, PeerInfo>>,
    configured_peers: Vec<String>,
}

impl AutonomyController {
    pub async fn new(pop_id: String, db_url: String) -> brivas_core::Result<Self> {
        // Configure known peers based on PoP tier
        let configured_peers = Self::get_configured_peers(&pop_id);
        
        Ok(Self {
            pop_id,
            db_url,
            mode: Arc::new(RwLock::new(AutonomyMode::Connected)),
            peer_status: Arc::new(DashMap::new()),
            configured_peers,
        })
    }
    
    fn get_configured_peers(pop_id: &str) -> Vec<String> {
        // Tier 1 PoPs peer with each other
        let tier1 = vec![
            "lagos-ng-1", "johannesburg-za-1", "london-uk-1",
            "frankfurt-de-1", "singapore-sg-1", "saopaulo-br-1"
        ];
        
        // Tier 2 peers with parent
        let tier2_parents = [
            ("nairobi-ke-1", "lagos-ng-1"),
            ("cairo-eg-1", "lagos-ng-1"),
            ("dubai-ae-1", "frankfurt-de-1"),
            ("mumbai-in-1", "singapore-sg-1"),
            ("tokyo-jp-1", "singapore-sg-1"),
        ];
        
        if tier1.contains(&pop_id) {
            tier1.iter()
                .filter(|&&p| p != pop_id)
                .map(|s| s.to_string())
                .collect()
        } else {
            tier2_parents.iter()
                .find(|(id, _)| *id == pop_id)
                .map(|(_, parent)| vec![parent.to_string()])
                .unwrap_or_default()
        }
    }
    
    /// Get current autonomy mode
    pub async fn current_mode(&self) -> AutonomyMode {
        *self.mode.read().await
    }
    
    /// Start autonomy monitoring
    pub async fn start(&self) -> brivas_core::Result<()> {
        let mut ticker = interval(Duration::from_secs(5));
        
        loop {
            ticker.tick().await;
            
            // Check peer connectivity
            for peer in &self.configured_peers {
                let status = self.check_peer_health(peer).await;
                self.peer_status.insert(peer.clone(), status);
            }
            
            // Update mode
            self.update_autonomy_mode().await;
        }
    }
    
    /// Check health of a peer PoP
    async fn check_peer_health(&self, peer_id: &str) -> PeerInfo {
        // In production, this would ping the peer's pop-controller
        // For now, simulate based on configured peers
        let start = std::time::Instant::now();
        
        // Placeholder: assume all peers healthy
        let healthy = true;
        let latency = start.elapsed().as_secs_f64() * 1000.0;
        
        PeerInfo {
            pop_id: peer_id.to_string(),
            healthy,
            last_seen: Utc::now(),
            latency_ms: Some(latency),
        }
    }
    
    /// Update mode based on peer connectivity
    async fn update_autonomy_mode(&self) {
        let total_peers = self.configured_peers.len();
        let healthy_peers = self.peer_status
            .iter()
            .filter(|e| e.healthy)
            .count();
        
        let new_mode = if total_peers == 0 {
            AutonomyMode::Isolated
        } else if healthy_peers == total_peers {
            AutonomyMode::Connected
        } else if healthy_peers > 0 {
            AutonomyMode::Degraded
        } else {
            AutonomyMode::Isolated
        };
        
        let mut mode = self.mode.write().await;
        if *mode != new_mode {
            tracing::warn!(
                pop_id = %self.pop_id,
                old_mode = ?*mode,
                new_mode = ?new_mode,
                "Autonomy mode changed"
            );
            *mode = new_mode;
        }
    }
    
    /// Get peer status
    pub fn get_peer_status(&self) -> Vec<PeerInfo> {
        self.peer_status.iter().map(|e| e.value().clone()).collect()
    }
}
