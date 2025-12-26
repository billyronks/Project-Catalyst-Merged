//! Capability Service - Check device RCS support

use brivas_rcs_sdk::capability::{DeviceCapability, RcsFeatures, RcsHub};
use chrono::{Duration, Utc};
use dashmap::DashMap;

use crate::config::RcsConfig;

pub struct CapabilityService {
    cache: DashMap<String, DeviceCapability>,
    cache_ttl: Duration,
    jibe_url: String,
}

impl CapabilityService {
    pub fn new(config: &RcsConfig) -> Self {
        Self {
            cache: DashMap::new(),
            cache_ttl: Duration::seconds(config.capability_cache_ttl_secs as i64),
            jibe_url: config.jibe_hub_url.clone(),
        }
    }

    /// Check if a phone number supports RCS
    pub async fn check(&self, phone_number: &str) -> DeviceCapability {
        // Check cache first
        if let Some(cached) = self.cache.get(phone_number) {
            if cached.is_valid() {
                return cached.clone();
            }
        }

        // Perform live check (simulated)
        let capability = self.live_check(phone_number).await;
        
        // Cache result
        self.cache.insert(phone_number.to_string(), capability.clone());
        
        capability
    }

    /// Batch check capabilities
    pub async fn batch_check(&self, phone_numbers: &[String]) -> Vec<DeviceCapability> {
        let mut results = Vec::with_capacity(phone_numbers.len());
        
        for phone in phone_numbers {
            results.push(self.check(phone).await);
        }
        
        results
    }

    async fn live_check(&self, phone_number: &str) -> DeviceCapability {
        // TODO: Actually call Jibe/Samsung/Mavenir APIs
        // For now, simulate based on number pattern
        
        let now = Utc::now();
        let rcs_enabled = !phone_number.starts_with("+1800"); // Toll-free numbers don't support RCS
        
        DeviceCapability {
            phone_number: phone_number.to_string(),
            rcs_enabled,
            carrier: Some(self.detect_carrier(phone_number)),
            carrier_rcs_hub: if rcs_enabled { Some(RcsHub::GoogleJibe) } else { None },
            features: if rcs_enabled { RcsFeatures::full() } else { RcsFeatures::default() },
            checked_at: now,
            cache_valid_until: now + self.cache_ttl,
        }
    }

    fn detect_carrier(&self, phone_number: &str) -> String {
        // Simplified carrier detection
        if phone_number.starts_with("+234803") || phone_number.starts_with("+234806") {
            "MTN Nigeria".to_string()
        } else if phone_number.starts_with("+234802") || phone_number.starts_with("+234808") {
            "Airtel Nigeria".to_string()
        } else if phone_number.starts_with("+1") {
            "US Carrier".to_string()
        } else if phone_number.starts_with("+44") {
            "UK Carrier".to_string()
        } else {
            "Unknown".to_string()
        }
    }
}
