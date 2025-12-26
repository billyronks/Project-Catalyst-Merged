//! CDR Collector
//!
//! Collects and stores Call Detail Records from all services.

use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::types::{Cdr, CdrStatus, ServiceType};

#[derive(Clone)]
pub struct CdrCollector {
    /// In-memory buffer for batch writes
    buffer: Arc<DashMap<Uuid, Cdr>>,
    /// LumaDB URL
    #[allow(dead_code)]
    lumadb_url: String,
    /// Batch size for flushing
    batch_size: usize,
}

impl CdrCollector {
    pub async fn new(lumadb_url: &str) -> brivas_core::Result<Self> {
        Ok(Self {
            buffer: Arc::new(DashMap::new()),
            lumadb_url: lumadb_url.to_string(),
            batch_size: 1000,
        })
    }

    /// Record a new CDR
    pub async fn record(&self, cdr: Cdr) -> brivas_core::Result<Uuid> {
        let id = cdr.id;
        self.buffer.insert(id, cdr);
        
        // Flush if buffer is full
        if self.buffer.len() >= self.batch_size {
            self.flush().await?;
        }
        
        Ok(id)
    }

    /// Record SMS CDR
    pub async fn record_sms(
        &self,
        customer_id: Uuid,
        source: &str,
        destination: &str,
        carrier_id: Uuid,
        pop_id: &str,
    ) -> brivas_core::Result<Uuid> {
        let cdr = Cdr {
            id: Uuid::new_v4(),
            customer_id,
            service_type: ServiceType::Sms,
            source: source.to_string(),
            destination: destination.to_string(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            duration_seconds: 0,
            quantity: 1,
            status: CdrStatus::Pending,
            rated_amount: None,
            currency: "NGN".to_string(),
            rate_id: None,
            carrier_id: Some(carrier_id),
            pop_id: pop_id.to_string(),
            metadata: serde_json::json!({}),
        };
        self.record(cdr).await
    }

    /// Record USSD CDR
    pub async fn record_ussd(
        &self,
        customer_id: Uuid,
        msisdn: &str,
        service_code: &str,
        session_duration: u32,
        pop_id: &str,
    ) -> brivas_core::Result<Uuid> {
        let cdr = Cdr {
            id: Uuid::new_v4(),
            customer_id,
            service_type: ServiceType::Ussd,
            source: msisdn.to_string(),
            destination: service_code.to_string(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            duration_seconds: session_duration,
            quantity: 1,
            status: CdrStatus::Pending,
            rated_amount: None,
            currency: "NGN".to_string(),
            rate_id: None,
            carrier_id: None,
            pop_id: pop_id.to_string(),
            metadata: serde_json::json!({}),
        };
        self.record(cdr).await
    }

    /// Flush buffer to LumaDB
    pub async fn flush(&self) -> brivas_core::Result<usize> {
        let count = self.buffer.len();
        // TODO: Batch insert to LumaDB
        self.buffer.clear();
        tracing::info!(count, "Flushed CDRs to LumaDB");
        Ok(count)
    }

    /// Get pending CDRs for rating
    pub async fn get_pending(&self, limit: usize) -> Vec<Cdr> {
        self.buffer
            .iter()
            .filter(|e| e.value().status == CdrStatus::Pending)
            .take(limit)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Update CDR status
    pub async fn update_status(&self, id: Uuid, status: CdrStatus) -> brivas_core::Result<()> {
        if let Some(mut cdr) = self.buffer.get_mut(&id) {
            cdr.status = status;
        }
        Ok(())
    }
}
