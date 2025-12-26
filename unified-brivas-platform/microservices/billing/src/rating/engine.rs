//! Rating Engine
//!
//! Real-time rating for CDRs with LCR support.

use dashmap::DashMap;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::{Cdr, CdrStatus, Rate, ServiceType, UnitType};

#[derive(Clone)]
pub struct RatingEngine {
    /// Rate cache
    rates: Arc<DashMap<String, Vec<Rate>>>,
    /// LumaDB URL
    #[allow(dead_code)]
    lumadb_url: String,
}

impl RatingEngine {
    pub async fn new(lumadb_url: &str) -> brivas_core::Result<Self> {
        let engine = Self {
            rates: Arc::new(DashMap::new()),
            lumadb_url: lumadb_url.to_string(),
        };
        engine.load_rates().await?;
        Ok(engine)
    }

    /// Load rates from LumaDB
    async fn load_rates(&self) -> brivas_core::Result<()> {
        // TODO: Load from LumaDB
        // For now, add default rates
        use rust_decimal_macros::dec;
        
        let sms_rate = Rate {
            id: Uuid::new_v4(),
            name: "SMS Nigeria".to_string(),
            service_type: ServiceType::Sms,
            destination_pattern: "234*".to_string(),
            unit_price: dec!(4.00),
            currency: "NGN".to_string(),
            unit_type: UnitType::PerMessage,
            minimum_charge: dec!(4.00),
            valid_from: chrono::Utc::now(),
            valid_until: None,
            priority: 100,
        };
        
        self.rates
            .entry("SMS:234".to_string())
            .or_insert_with(Vec::new)
            .push(sms_rate);

        Ok(())
    }

    /// Rate a CDR
    pub async fn rate(&self, cdr: &mut Cdr) -> brivas_core::Result<Decimal> {
        let rate = self.find_rate(cdr.service_type, &cdr.destination)?;
        
        let amount = match rate.unit_type {
            UnitType::PerMessage => rate.unit_price * Decimal::from(cdr.quantity),
            UnitType::PerSecond => rate.unit_price * Decimal::from(cdr.duration_seconds),
            UnitType::PerMinute => {
                let minutes = (cdr.duration_seconds as f64 / 60.0).ceil();
                rate.unit_price * Decimal::from(minutes as u32)
            }
            UnitType::PerSession => rate.unit_price,
            UnitType::PerMb => rate.unit_price * Decimal::from(cdr.quantity),
        };

        let final_amount = amount.max(rate.minimum_charge);
        
        cdr.rated_amount = Some(final_amount);
        cdr.rate_id = Some(rate.id);
        cdr.status = CdrStatus::Rated;

        Ok(final_amount)
    }

    /// Find the best rate for a destination
    fn find_rate(&self, service_type: ServiceType, destination: &str) -> brivas_core::Result<Rate> {
        // Try to find matching rate
        let prefix = format!("{:?}:{}", service_type, &destination[..3.min(destination.len())]);
        
        if let Some(rates) = self.rates.get(&prefix) {
            if let Some(rate) = rates.first() {
                return Ok(rate.clone());
            }
        }

        // Default rate
        use rust_decimal_macros::dec;
        Ok(Rate {
            id: Uuid::nil(),
            name: "Default".to_string(),
            service_type,
            destination_pattern: "*".to_string(),
            unit_price: dec!(5.00),
            currency: "NGN".to_string(),
            unit_type: UnitType::PerMessage,
            minimum_charge: dec!(5.00),
            valid_from: chrono::Utc::now(),
            valid_until: None,
            priority: 0,
        })
    }

    /// Rate multiple CDRs
    pub async fn rate_batch(&self, cdrs: &mut [Cdr]) -> brivas_core::Result<Decimal> {
        let mut total = Decimal::ZERO;
        for cdr in cdrs.iter_mut() {
            total += self.rate(cdr).await?;
        }
        Ok(total)
    }

    /// Add or update a rate
    pub async fn upsert_rate(&self, rate: Rate) -> brivas_core::Result<()> {
        let key = format!("{:?}:{}", rate.service_type, &rate.destination_pattern[..3.min(rate.destination_pattern.len())]);
        self.rates
            .entry(key)
            .or_insert_with(Vec::new)
            .push(rate);
        Ok(())
    }
}
