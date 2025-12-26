//! Peer PoP Monitor
//!
//! Monitors connectivity to peer PoPs and manages mesh health.

use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use tokio::time::interval;

/// Peer PoP connection status
#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub peer_id: String,
    pub endpoint: String,
    pub connected: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub latency_ms: f64,
    pub failed_attempts: u32,
}

/// Peer monitor for mesh connectivity
pub struct PeerMonitor {
    pop_id: String,
    peers: HashMap<String, PeerConnection>,
    heartbeat_interval: Duration,
}

impl PeerMonitor {
    pub fn new(pop_id: String) -> Self {
        Self {
            pop_id,
            peers: HashMap::new(),
            heartbeat_interval: Duration::from_secs(5),
        }
    }
    
    /// Add a peer to monitor
    pub fn add_peer(&mut self, peer_id: String, endpoint: String) {
        self.peers.insert(peer_id.clone(), PeerConnection {
            peer_id,
            endpoint,
            connected: false,
            last_heartbeat: Utc::now(),
            latency_ms: 0.0,
            failed_attempts: 0,
        });
    }
    
    /// Start monitoring peers
    pub async fn start(&mut self) {
        let mut ticker = interval(self.heartbeat_interval);
        
        loop {
            ticker.tick().await;
            
            // Collect endpoints first to avoid borrow issues
            let peer_endpoints: Vec<(String, String)> = self.peers
                .iter()
                .map(|(id, conn)| (id.clone(), conn.endpoint.clone()))
                .collect();
            
            for (peer_id, endpoint) in peer_endpoints {
                let start = std::time::Instant::now();
                
                // Attempt heartbeat
                let result = Self::send_heartbeat_static(&endpoint).await;
                
                if let Some(conn) = self.peers.get_mut(&peer_id) {
                    match result {
                        Ok(_) => {
                            conn.connected = true;
                            conn.last_heartbeat = Utc::now();
                            conn.latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                            conn.failed_attempts = 0;
                        }
                        Err(_) => {
                            conn.failed_attempts += 1;
                            if conn.failed_attempts >= 3 {
                                conn.connected = false;
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Send heartbeat to peer (static version)
    async fn send_heartbeat_static(endpoint: &str) -> Result<(), std::io::Error> {
        match tokio::net::TcpStream::connect(endpoint).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Send heartbeat to peer
    async fn send_heartbeat(&self, endpoint: &str) -> Result<(), std::io::Error> {
        // TCP connect as basic health check
        match tokio::net::TcpStream::connect(endpoint).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    /// Get connected peer count
    pub fn connected_count(&self) -> usize {
        self.peers.values().filter(|p| p.connected).count()
    }
    
    /// Get all peer statuses
    pub fn get_peer_status(&self) -> Vec<&PeerConnection> {
        self.peers.values().collect()
    }
    
    /// Check if specific peer is connected
    pub fn is_peer_connected(&self, peer_id: &str) -> bool {
        self.peers.get(peer_id).map(|p| p.connected).unwrap_or(false)
    }
}
