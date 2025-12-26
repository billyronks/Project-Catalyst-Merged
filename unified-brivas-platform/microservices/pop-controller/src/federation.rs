//! LumaDB Federation Configuration
//!
//! Configures multi-PoP data replication with CRDT-based conflict resolution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Federation configuration for LumaDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// This PoP's unique identifier
    pub node_id: String,
    
    /// Replication topology
    pub topology: ReplicationTopology,
    
    /// Conflict resolution per collection
    pub conflict_resolution: HashMap<String, ConflictResolution>,
    
    /// Replication settings
    pub replication: ReplicationSettings,
    
    /// Collections requiring synchronous replication
    pub sync_collections: Vec<String>,
    
    /// Collections that are local-only (no replication)
    pub local_only_collections: Vec<String>,
}

/// Replication topology types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationTopology {
    /// All Tier-1 PoPs sync with each other
    FullMesh,
    
    /// Star topology with central hub
    Star { hub: String },
    
    /// Hierarchical with parent node
    Hierarchical { parent: String },
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Last writer wins with optional vector clock
    LastWriterWins { use_vector_clock: bool },
    
    /// CRDT G-Counter for additive operations
    CrdtCounter,
    
    /// CRDT LWW-Register
    CrdtLwwRegister,
    
    /// Custom handler
    Custom { handler: String },
}

/// Replication settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationSettings {
    /// Sync interval (ms)
    pub sync_interval_ms: u64,
    
    /// Batch size
    pub batch_size: usize,
    
    /// Compression algorithm
    pub compression: String,
    
    /// Encryption enabled
    pub encryption: bool,
    
    /// Retry configuration
    pub max_retries: u32,
    pub retry_backoff_ms: u64,
}

impl Default for ReplicationSettings {
    fn default() -> Self {
        Self {
            sync_interval_ms: 100,
            batch_size: 1000,
            compression: "zstd".to_string(),
            encryption: true,
            max_retries: 5,
            retry_backoff_ms: 1000,
        }
    }
}

impl FederationConfig {
    /// Create configuration for a PoP
    pub fn for_pop(pop_id: &str, tier: PopTier) -> Self {
        let topology = match tier {
            PopTier::Tier1 => ReplicationTopology::FullMesh,
            PopTier::Tier2 { parent } => ReplicationTopology::Hierarchical { parent },
            PopTier::Tier3 { parent } => ReplicationTopology::Star { hub: parent },
        };
        
        let mut conflict_resolution = HashMap::new();
        
        // Messages: LWW with vector clock
        conflict_resolution.insert(
            "messages".to_string(),
            ConflictResolution::LastWriterWins { use_vector_clock: true },
        );
        
        // Balances: CRDT counter
        conflict_resolution.insert(
            "account_balances".to_string(),
            ConflictResolution::CrdtCounter,
        );
        
        // Users: LWW
        conflict_resolution.insert(
            "users".to_string(),
            ConflictResolution::LastWriterWins { use_vector_clock: true },
        );
        
        // Sessions: custom merge
        conflict_resolution.insert(
            "sessions".to_string(),
            ConflictResolution::Custom { handler: "session_merge".to_string() },
        );
        
        Self {
            node_id: pop_id.to_string(),
            topology,
            conflict_resolution,
            replication: ReplicationSettings::default(),
            sync_collections: vec![
                "billing_transactions".to_string(),
                "number_portability".to_string(),
            ],
            local_only_collections: vec![
                "local_cache".to_string(),
                "temp_sessions".to_string(),
                "rate_limit_counters".to_string(),
            ],
        }
    }
    
    /// Get sync interval as Duration
    pub fn sync_interval(&self) -> Duration {
        Duration::from_millis(self.replication.sync_interval_ms)
    }
}

/// PoP tier classification
#[derive(Debug, Clone)]
pub enum PopTier {
    Tier1,
    Tier2 { parent: String },
    Tier3 { parent: String },
}

impl PopTier {
    pub fn from_pop_id(pop_id: &str) -> Self {
        match pop_id {
            "lagos-ng-1" | "johannesburg-za-1" | "london-uk-1" |
            "frankfurt-de-1" | "singapore-sg-1" | "saopaulo-br-1" => PopTier::Tier1,
            
            "nairobi-ke-1" => PopTier::Tier2 { parent: "lagos-ng-1".to_string() },
            "cairo-eg-1" => PopTier::Tier2 { parent: "lagos-ng-1".to_string() },
            "dubai-ae-1" => PopTier::Tier2 { parent: "frankfurt-de-1".to_string() },
            "mumbai-in-1" => PopTier::Tier2 { parent: "singapore-sg-1".to_string() },
            "tokyo-jp-1" => PopTier::Tier2 { parent: "singapore-sg-1".to_string() },
            
            _ => PopTier::Tier3 { parent: "lagos-ng-1".to_string() },
        }
    }
}
