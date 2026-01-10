//! Unified Messaging Hub - Multi-Protocol Message Orchestration
//!
//! Provides unified API for:
//! - SMS (via SMSC)
//! - RCS (Rich Communication Services)
//! - WhatsApp Business API
//! - Telegram Bot API
//! - Push Notifications (FCM/APNS)
//! - Email (SMTP/API)
//!
//! Features:
//! - Intelligent channel selection
//! - Automatic fallback routing
//! - Message templating
//! - Real-time delivery tracking

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Message channel types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    Sms,
    Rcs,
    WhatsApp,
    Telegram,
    PushNotification,
    Email,
    InApp,
}

impl Channel {
    pub fn priority(&self) -> u8 {
        match self {
            Channel::Rcs => 1,
            Channel::WhatsApp => 2,
            Channel::Sms => 3,
            Channel::Telegram => 4,
            Channel::PushNotification => 5,
            Channel::Email => 6,
            Channel::InApp => 7,
        }
    }
}

/// Unified message representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub recipient: String,
    pub content: MessageContent,
    pub preferred_channels: Vec<Channel>,
    pub fallback_enabled: bool,
    pub schedule_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: std::collections::HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Template { id: String, variables: std::collections::HashMap<String, String> },
    Rich { text: String, media_url: Option<String>, buttons: Vec<MessageButton> },
    Binary { data: Vec<u8>, mime_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageButton {
    pub label: String,
    pub action: ButtonAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ButtonAction {
    Url(String),
    Callback(String),
    Phone(String),
}

/// Delivery status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryStatus {
    pub message_id: Uuid,
    pub channel: Channel,
    pub status: Status,
    pub carrier_message_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub attempts: u8,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Queued,
    Sent,
    Delivered,
    Read,
    Failed,
    Expired,
}

/// Channel adapter trait for pluggable channel implementations
#[async_trait::async_trait]
pub trait ChannelAdapter: Send + Sync {
    fn channel(&self) -> Channel;
    async fn send(&self, message: &UnifiedMessage) -> Result<DeliveryStatus, ChannelError>;
    async fn check_status(&self, carrier_id: &str) -> Result<Status, ChannelError>;
    fn is_available(&self, recipient: &str) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("Channel unavailable: {0}")]
    Unavailable(String),
    #[error("Rate limited: retry after {0}s")]
    RateLimited(u32),
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
}

/// High-performance message orchestrator
pub struct MessageOrchestrator {
    adapters: Arc<DashMap<Channel, Arc<dyn ChannelAdapter>>>,
    message_queue: mpsc::Sender<UnifiedMessage>,
    status_cache: Arc<DashMap<Uuid, DeliveryStatus>>,
    analytics: Arc<UmhAnalytics>,
}

impl MessageOrchestrator {
    pub async fn new(analytics: Arc<UmhAnalytics>) -> Self {
        let (tx, mut rx) = mpsc::channel::<UnifiedMessage>(100_000);
        let adapters: Arc<DashMap<Channel, Arc<dyn ChannelAdapter>>> = Arc::new(DashMap::new());
        let status_cache = Arc::new(DashMap::new());

        // Spawn message processor
        let adapters_clone = adapters.clone();
        let status_clone = status_cache.clone();
        let analytics_clone = analytics.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                Self::process_message(&adapters_clone, &status_clone, &analytics_clone, msg).await;
            }
        });

        Self {
            adapters,
            message_queue: tx,
            status_cache,
            analytics,
        }
    }

    /// Register a channel adapter
    pub fn register_adapter(&self, adapter: Arc<dyn ChannelAdapter>) {
        self.adapters.insert(adapter.channel(), adapter);
    }

    /// Send a message through the best available channel
    pub async fn send(&self, message: UnifiedMessage) -> Result<Uuid, ChannelError> {
        let id = message.id;
        
        // Initialize status
        self.status_cache.insert(id, DeliveryStatus {
            message_id: id,
            channel: message.preferred_channels.first().copied().unwrap_or(Channel::Sms),
            status: Status::Pending,
            carrier_message_id: None,
            error_code: None,
            error_message: None,
            attempts: 0,
            updated_at: Utc::now(),
        });

        // Queue for processing
        self.message_queue.send(message).await.map_err(|e| {
            ChannelError::SendFailed(format!("Queue full: {}", e))
        })?;

        Ok(id)
    }

    /// Get delivery status
    pub fn get_status(&self, message_id: &Uuid) -> Option<DeliveryStatus> {
        self.status_cache.get(message_id).map(|s| s.clone())
    }

    async fn process_message(
        adapters: &DashMap<Channel, Arc<dyn ChannelAdapter>>,
        status_cache: &DashMap<Uuid, DeliveryStatus>,
        analytics: &UmhAnalytics,
        message: UnifiedMessage,
    ) {
        let start = std::time::Instant::now();
        let mut last_error = None;

        // Try channels in order of preference
        for channel in &message.preferred_channels {
            if let Some(adapter) = adapters.get(channel) {
                // Check if channel can reach recipient
                if !adapter.is_available(&message.recipient) {
                    debug!(?channel, "Channel not available for recipient");
                    continue;
                }

                match adapter.send(&message).await {
                    Ok(status) => {
                        status_cache.insert(message.id, status.clone());
                        
                        // Record analytics
                        analytics.record_delivery(
                            message.id,
                            *channel,
                            status.status,
                            start.elapsed().as_millis() as i32,
                        ).await;

                        info!(
                            message_id = %message.id,
                            ?channel,
                            latency_ms = start.elapsed().as_millis(),
                            "Message sent successfully"
                        );
                        return;
                    }
                    Err(e) => {
                        warn!(?channel, error = %e, "Channel failed, trying next");
                        last_error = Some(e);
                        
                        if !message.fallback_enabled {
                            break;
                        }
                    }
                }
            }
        }

        // All channels failed
        let error = last_error.unwrap_or(ChannelError::Unavailable("No channels available".into()));
        status_cache.insert(message.id, DeliveryStatus {
            message_id: message.id,
            channel: message.preferred_channels.first().copied().unwrap_or(Channel::Sms),
            status: Status::Failed,
            carrier_message_id: None,
            error_code: Some("ALL_CHANNELS_FAILED".into()),
            error_message: Some(error.to_string()),
            attempts: message.preferred_channels.len() as u8,
            updated_at: Utc::now(),
        });

        analytics.record_failure(message.id, &error.to_string()).await;
    }
}

/// UMH Analytics with QuestDB
pub struct UmhAnalytics {
    questdb: Arc<tokio_postgres::Client>,
}

impl UmhAnalytics {
    pub async fn new(questdb_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (client, connection) = tokio_postgres::connect(questdb_url, tokio_postgres::NoTls).await?;
        
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("QuestDB connection error: {}", e);
            }
        });

        // Create UMH analytics tables
        client.execute(
            "CREATE TABLE IF NOT EXISTS umh_messages (
                message_id SYMBOL,
                channel SYMBOL,
                status SYMBOL,
                latency_ms INT,
                timestamp TIMESTAMP
            ) TIMESTAMP(timestamp) PARTITION BY DAY WAL",
            &[],
        ).await.ok();

        Ok(Self {
            questdb: Arc::new(client),
        })
    }

    pub async fn record_delivery(&self, message_id: Uuid, channel: Channel, status: Status, latency_ms: i32) {
        let msg_id = message_id.to_string();
        let channel_str = format!("{:?}", channel);
        let status_str = format!("{:?}", status);
        let db = self.questdb.clone();

        tokio::spawn(async move {
            db.execute(
                "INSERT INTO umh_messages VALUES ($1, $2, $3, $4, now())",
                &[&msg_id, &channel_str, &status_str, &latency_ms],
            ).await.ok();
        });
    }

    pub async fn record_failure(&self, message_id: Uuid, error: &str) {
        let msg_id = message_id.to_string();
        let db = self.questdb.clone();
        let error = error.to_string();

        tokio::spawn(async move {
            db.execute(
                "INSERT INTO umh_messages VALUES ($1, 'FAILED', $2, -1, now())",
                &[&msg_id, &error],
            ).await.ok();
        });
    }

    pub async fn get_channel_stats(&self, hours: i32) -> Result<Vec<(String, i64, f64)>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = self.questdb.query(
            "SELECT channel, count(*) as volume, 
                    sum(CASE WHEN status = 'Delivered' THEN 1.0 ELSE 0.0 END) / count(*) * 100 as delivery_rate
             FROM umh_messages 
             WHERE timestamp > dateadd('h', $1, now())
             GROUP BY channel",
            &[&(-hours)],
        ).await?;

        Ok(rows.iter().map(|r| (r.get(0), r.get(1), r.get(2))).collect())
    }
}

/// SMS Channel Adapter (connects to SMSC)
pub struct SmsAdapter {
    smsc_client: Arc<tokio::sync::Mutex<SmscClient>>,
}

struct SmscClient {
    // Placeholder for SMSC connection
}

#[async_trait::async_trait]
impl ChannelAdapter for SmsAdapter {
    fn channel(&self) -> Channel {
        Channel::Sms
    }

    async fn send(&self, message: &UnifiedMessage) -> Result<DeliveryStatus, ChannelError> {
        // Extract text content
        let text = match &message.content {
            MessageContent::Text(t) => t.clone(),
            MessageContent::Template { id, variables } => format!("Template: {} {:?}", id, variables),
            MessageContent::Rich { text, .. } => text.clone(),
            _ => return Err(ChannelError::SendFailed("SMS only supports text".into())),
        };

        // Send via SMSC (placeholder)
        let carrier_id = Uuid::new_v4().to_string();

        Ok(DeliveryStatus {
            message_id: message.id,
            channel: Channel::Sms,
            status: Status::Sent,
            carrier_message_id: Some(carrier_id),
            error_code: None,
            error_message: None,
            attempts: 1,
            updated_at: Utc::now(),
        })
    }

    async fn check_status(&self, _carrier_id: &str) -> Result<Status, ChannelError> {
        Ok(Status::Delivered)
    }

    fn is_available(&self, recipient: &str) -> bool {
        // Check if recipient is a valid phone number
        recipient.starts_with('+') || recipient.chars().all(|c| c.is_ascii_digit())
    }
}
