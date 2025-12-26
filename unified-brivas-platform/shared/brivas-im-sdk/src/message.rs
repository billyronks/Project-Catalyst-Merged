//! Message types for Instant Messaging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Message entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: MessageContent,
    pub message_type: MessageType,
    pub reply_to: Option<Uuid>,
    pub forwarded_from: Option<ForwardedInfo>,
    pub reactions: Vec<Reaction>,
    pub read_by: Vec<ReadReceipt>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_for: Vec<Uuid>,
    pub deleted_for_everyone: bool,
    pub encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Message content variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text {
        text: String,
        mentions: Vec<Mention>,
    },
    Image {
        url: String,
        thumbnail_url: String,
        width: u32,
        height: u32,
        caption: Option<String>,
    },
    Video {
        url: String,
        thumbnail_url: String,
        duration_seconds: u32,
        caption: Option<String>,
    },
    Audio {
        url: String,
        duration_seconds: u32,
        waveform: Option<Vec<u8>>,
    },
    File {
        url: String,
        filename: String,
        mime_type: String,
        size_bytes: u64,
    },
    Location {
        latitude: f64,
        longitude: f64,
        name: Option<String>,
        address: Option<String>,
    },
    Contact {
        name: String,
        phone_numbers: Vec<String>,
    },
    Sticker {
        pack_id: String,
        sticker_id: String,
        url: String,
    },
    System {
        event: SystemEvent,
    },
}

/// Message type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Image,
    Video,
    Audio,
    File,
    Location,
    Contact,
    Sticker,
    System,
}

/// System event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemEvent {
    ConversationCreated,
    ParticipantAdded { user_id: Uuid, added_by: Uuid },
    ParticipantRemoved { user_id: Uuid, removed_by: Uuid },
    ParticipantLeft { user_id: Uuid },
    ConversationRenamed { old_name: String, new_name: String },
    AvatarChanged,
}

/// Forwarded message info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardedInfo {
    pub original_message_id: Uuid,
    pub original_sender_id: Uuid,
    pub original_conversation_id: Uuid,
    pub forward_count: u32,
}

/// Message reaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Read receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadReceipt {
    pub user_id: Uuid,
    pub read_at: DateTime<Utc>,
}

/// User mention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub user_id: Uuid,
    pub offset: u32,
    pub length: u32,
}

impl Message {
    /// Create a new text message
    pub fn new_text(conversation_id: Uuid, sender_id: Uuid, text: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            sender_id,
            content: MessageContent::Text { text, mentions: vec![] },
            message_type: MessageType::Text,
            reply_to: None,
            forwarded_from: None,
            reactions: vec![],
            read_by: vec![],
            edited_at: None,
            deleted_for: vec![],
            deleted_for_everyone: false,
            encrypted: false,
            created_at: now,
            updated_at: now,
        }
    }
}
