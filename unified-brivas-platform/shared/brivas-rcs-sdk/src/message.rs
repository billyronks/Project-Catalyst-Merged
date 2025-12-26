//! RCS Message types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rich_card::RichCard;
use crate::suggestion::Suggestion;

/// RCS Message entity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsMessage {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub recipient_phone: String,
    pub message_type: RcsMessageType,
    pub content: RcsMessageContent,
    pub suggestions: Vec<Suggestion>,
    pub status: RcsMessageStatus,
    pub hub_message_id: Option<String>,
    pub hub: Option<String>,
    pub fallback_to_sms: bool,
    pub sms_message_id: Option<Uuid>,
    pub error_details: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
}

/// RCS message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RcsMessageType {
    Text,
    RichCard,
    Carousel,
    File,
}

/// RCS message content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RcsMessageContent {
    Text { text: String },
    RichCard { rich_card: RichCard },
    File { 
        url: String, 
        filename: String, 
        mime_type: String 
    },
}

/// RCS message status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RcsMessageStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
    FallbackToSms,
}

impl RcsMessage {
    /// Create a new text message
    pub fn new_text(agent_id: Uuid, recipient_phone: String, text: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_id,
            conversation_id: None,
            recipient_phone,
            message_type: RcsMessageType::Text,
            content: RcsMessageContent::Text { text },
            suggestions: vec![],
            status: RcsMessageStatus::Pending,
            hub_message_id: None,
            hub: None,
            fallback_to_sms: false,
            sms_message_id: None,
            error_details: None,
            created_at: now,
            sent_at: None,
            delivered_at: None,
            read_at: None,
        }
    }

    /// Create a new rich card message
    pub fn new_rich_card(agent_id: Uuid, recipient_phone: String, rich_card: RichCard) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_id,
            conversation_id: None,
            recipient_phone,
            message_type: RcsMessageType::RichCard,
            content: RcsMessageContent::RichCard { rich_card },
            suggestions: vec![],
            status: RcsMessageStatus::Pending,
            hub_message_id: None,
            hub: None,
            fallback_to_sms: false,
            sms_message_id: None,
            error_details: None,
            created_at: now,
            sent_at: None,
            delivered_at: None,
            read_at: None,
        }
    }
}
