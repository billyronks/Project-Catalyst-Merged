//! Device capability checking for RCS

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Device RCS capability
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCapability {
    pub phone_number: String,
    pub rcs_enabled: bool,
    pub carrier: Option<String>,
    pub carrier_rcs_hub: Option<RcsHub>,
    pub features: RcsFeatures,
    pub checked_at: DateTime<Utc>,
    pub cache_valid_until: DateTime<Utc>,
}

/// RCS Hub providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RcsHub {
    GoogleJibe,
    SamsungRnc,
    Mavenir,
    DirectCarrier,
}

/// RCS feature capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RcsFeatures {
    pub rich_card: bool,
    pub carousel: bool,
    pub file_transfer: bool,
    pub file_transfer_max_size_mb: u32,
    pub suggested_replies: bool,
    pub suggested_actions: bool,
    pub location_sharing: bool,
    pub typing_indicators: bool,
    pub read_receipts: bool,
    pub revocation: bool,
}

/// Message channel after capability check
#[derive(Debug, Clone)]
pub enum MessageChannel {
    Rcs {
        hub: Option<RcsHub>,
        features: RcsFeatures,
    },
    SmsFallback,
}

impl DeviceCapability {
    /// Check if still valid (not expired)
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.cache_valid_until
    }

    /// Get the appropriate message channel
    pub fn to_channel(&self) -> MessageChannel {
        if self.rcs_enabled {
            MessageChannel::Rcs {
                hub: self.carrier_rcs_hub,
                features: self.features.clone(),
            }
        } else {
            MessageChannel::SmsFallback
        }
    }
}

impl RcsFeatures {
    /// Full RCS feature set
    pub fn full() -> Self {
        Self {
            rich_card: true,
            carousel: true,
            file_transfer: true,
            file_transfer_max_size_mb: 100,
            suggested_replies: true,
            suggested_actions: true,
            location_sharing: true,
            typing_indicators: true,
            read_receipts: true,
            revocation: true,
        }
    }

    /// Basic RCS feature set
    pub fn basic() -> Self {
        Self {
            rich_card: true,
            carousel: false,
            file_transfer: true,
            file_transfer_max_size_mb: 10,
            suggested_replies: true,
            suggested_actions: false,
            location_sharing: false,
            typing_indicators: true,
            read_receipts: true,
            revocation: false,
        }
    }
}
