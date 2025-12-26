//! Missed Call Alert Service
//!
//! Detects missed calls and triggers notifications via SMS/WhatsApp.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use dashmap::DashMap;

use crate::VoiceIvrConfig;

/// Missed Call Registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissedCallRegistration {
    pub number: String,
    pub callback_url: String,
    pub notification_channel: NotificationChannel,
    pub metadata: Option<serde_json::Value>,
}

/// Notification channel for missed calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Sms,
    WhatsApp,
    Webhook,
    All,
}

/// Missed Call Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissedCallEvent {
    pub event_id: String,
    pub registered_number: String,
    pub caller_number: String,
    pub call_time: chrono::DateTime<Utc>,
    pub ring_duration_seconds: u32,
    pub notification_sent: bool,
}

/// Missed Call Service
pub struct MissedCallService {
    #[allow(dead_code)]
    config: VoiceIvrConfig,
    registrations: Arc<DashMap<String, MissedCallRegistration>>,
    events: Arc<DashMap<String, MissedCallEvent>>,
}

impl MissedCallService {
    pub async fn new(config: &VoiceIvrConfig) -> brivas_core::Result<Self> {
        Ok(Self {
            config: config.clone(),
            registrations: Arc::new(DashMap::new()),
            events: Arc::new(DashMap::new()),
        })
    }

    /// Register a number for missed call alerts
    pub async fn register(&self, registration: MissedCallRegistration) -> Result<String, MissedCallError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.registrations.insert(registration.number.clone(), registration);
        
        tracing::info!(id = %id, "Missed call number registered");
        Ok(id)
    }

    /// Process incoming missed call event (called by OpenSIPS)
    pub async fn process_missed_call(
        &self,
        called_number: &str,
        caller_number: &str,
        ring_duration: u32,
    ) -> Result<(), MissedCallError> {
        // Check if this number is registered
        let registration = self.registrations
            .get(called_number)
            .ok_or(MissedCallError::NotRegistered)?;

        let event = MissedCallEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            registered_number: called_number.to_string(),
            caller_number: caller_number.to_string(),
            call_time: Utc::now(),
            ring_duration_seconds: ring_duration,
            notification_sent: false,
        };

        // Send notification based on channel
        match &registration.notification_channel {
            NotificationChannel::Sms => {
                self.send_sms_notification(&event).await?;
            }
            NotificationChannel::WhatsApp => {
                self.send_whatsapp_notification(&event).await?;
            }
            NotificationChannel::Webhook => {
                self.send_webhook_notification(&registration.callback_url, &event).await?;
            }
            NotificationChannel::All => {
                self.send_sms_notification(&event).await.ok();
                self.send_whatsapp_notification(&event).await.ok();
                self.send_webhook_notification(&registration.callback_url, &event).await.ok();
            }
        }

        // Store event
        self.events.insert(event.event_id.clone(), event);

        Ok(())
    }

    async fn send_sms_notification(&self, event: &MissedCallEvent) -> Result<(), MissedCallError> {
        // TODO: Integrate with SMSC
        tracing::info!(
            caller = %event.caller_number,
            "SMS notification sent for missed call"
        );
        Ok(())
    }

    async fn send_whatsapp_notification(&self, event: &MissedCallEvent) -> Result<(), MissedCallError> {
        // TODO: Integrate with Unified Messaging Hub
        tracing::info!(
            caller = %event.caller_number,
            "WhatsApp notification sent for missed call"
        );
        Ok(())
    }

    async fn send_webhook_notification(
        &self,
        callback_url: &str,
        _event: &MissedCallEvent,
    ) -> Result<(), MissedCallError> {
        // TODO: Send HTTP POST
        tracing::info!(
            callback_url = %callback_url,
            "Webhook notification sent for missed call"
        );
        Ok(())
    }

    /// Get events for a registered number
    pub async fn get_events(&self, number: &str) -> Vec<MissedCallEvent> {
        self.events
            .iter()
            .filter(|e| e.registered_number == number)
            .map(|e| e.value().clone())
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MissedCallError {
    #[error("Number not registered")]
    NotRegistered,
    
    #[error("Notification failed: {0}")]
    NotificationFailed(String),
}
