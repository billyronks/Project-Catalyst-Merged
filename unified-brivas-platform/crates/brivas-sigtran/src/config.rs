//! SIGTRAN configuration

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

/// Complete SIGTRAN configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigtranConfig {
    /// SCTP configuration
    pub sctp: SctpConfig,
    /// M3UA configuration
    pub m3ua: M3uaConfig,
    /// SCCP configuration
    pub sccp: SccpConfig,
    /// MAP configuration
    pub map: MapConfig,
}

/// SCTP layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SctpConfig {
    /// Local bind address
    pub local_address: String,
    /// Remote peer address
    pub remote_address: String,
    /// Port (default 2905 for M3UA)
    pub port: u16,
    /// Number of streams
    pub streams: u16,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Max retransmissions
    pub max_retrans: u32,
    /// RTO initial (ms)
    pub rto_initial_ms: u64,
    /// RTO min (ms)
    pub rto_min_ms: u64,
    /// RTO max (ms)
    pub rto_max_ms: u64,
}

/// M3UA layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M3uaConfig {
    /// Local point code
    pub point_code: u32,
    /// Network indicator
    pub network_indicator: u8,
    /// Routing contexts (optional)
    pub routing_contexts: Option<Vec<u32>>,
    /// Network appearance (optional)
    pub network_appearance: Option<u32>,
    /// Traffic mode
    pub traffic_mode: String,
    /// ASP identifier
    pub asp_identifier: Option<String>,
}

/// SCCP layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SccpConfig {
    /// Local subsystem number
    pub local_ssn: u8,
    /// Local Global Title
    pub global_title: Option<String>,
    /// Global Title Indicator (1-4)
    pub gti: u8,
    /// Translation type
    pub translation_type: u8,
    /// Numbering plan
    pub numbering_plan: u8,
    /// Nature of address
    pub nature_of_address: u8,
    /// Point code included in address
    pub include_pc: bool,
}

/// MAP layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    /// HLR Global Title
    pub hlr_gt: String,
    /// MSC Global Title
    pub msc_gt: String,
    /// Service centre address
    pub service_centre_address: String,
    /// USSD Service Code
    pub ussd_service_code: Option<String>,
    /// Operation timeout (ms)
    pub operation_timeout_ms: u64,
}

impl Default for SigtranConfig {
    fn default() -> Self {
        Self {
            sctp: SctpConfig::default(),
            m3ua: M3uaConfig::default(),
            sccp: SccpConfig::default(),
            map: MapConfig::default(),
        }
    }
}

impl Default for SctpConfig {
    fn default() -> Self {
        Self {
            local_address: "0.0.0.0".to_string(),
            remote_address: "127.0.0.1".to_string(),
            port: 2905,
            streams: 2,
            heartbeat_interval_ms: 30000,
            max_retrans: 10,
            rto_initial_ms: 3000,
            rto_min_ms: 1000,
            rto_max_ms: 60000,
        }
    }
}

impl Default for M3uaConfig {
    fn default() -> Self {
        Self {
            point_code: 1001,
            network_indicator: 2, // National
            routing_contexts: None,
            network_appearance: None,
            traffic_mode: "override".to_string(),
            asp_identifier: None,
        }
    }
}

impl Default for SccpConfig {
    fn default() -> Self {
        Self {
            local_ssn: 8, // SMSC
            global_title: None,
            gti: 4,
            translation_type: 0,
            numbering_plan: 1, // E.164
            nature_of_address: 4, // International
            include_pc: false,
        }
    }
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            hlr_gt: "".to_string(),
            msc_gt: "".to_string(),
            service_centre_address: "".to_string(),
            ussd_service_code: None,
            operation_timeout_ms: 30000,
        }
    }
}

impl SigtranConfig {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, crate::SigtranError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::SigtranError::Config(e.to_string()))?;
        
        serde_json::from_str(&content)
            .map_err(|e| crate::SigtranError::Config(e.to_string()))
    }

    /// Get heartbeat interval as Duration
    pub fn heartbeat_interval(&self) -> Duration {
        Duration::from_millis(self.sctp.heartbeat_interval_ms)
    }

    /// Get operation timeout as Duration
    pub fn operation_timeout(&self) -> Duration {
        Duration::from_millis(self.map.operation_timeout_ms)
    }
}


