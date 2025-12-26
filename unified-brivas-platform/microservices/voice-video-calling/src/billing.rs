//! Billing Integration
//!
//! Voice call billing integration with billing-service.

use crate::VoiceIvrConfig;

/// Billing integration (placeholder)
pub struct BillingIntegration {
    #[allow(dead_code)]
    config: VoiceIvrConfig,
}

impl BillingIntegration {
    pub async fn new(config: &VoiceIvrConfig) -> brivas_core::Result<Self> {
        Ok(Self { config: config.clone() })
    }
}
