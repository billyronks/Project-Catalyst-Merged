//! Presence and typing indicator types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User presence status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceStatus {
    pub user_id: Uuid,
    pub status: Status,
    pub custom_status: Option<String>,
    pub custom_status_expires_at: Option<DateTime<Utc>>,
    pub last_seen_at: DateTime<Utc>,
    pub device_id: Option<String>,
}

/// Status types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Online,
    Away,
    Busy,
    Offline,
}

/// Typing indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicator {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub started_at: DateTime<Utc>,
}

/// Presence update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub user_id: Uuid,
    pub old_status: Status,
    pub new_status: Status,
    pub timestamp: DateTime<Utc>,
}

impl PresenceStatus {
    pub fn online(user_id: Uuid) -> Self {
        Self {
            user_id,
            status: Status::Online,
            custom_status: None,
            custom_status_expires_at: None,
            last_seen_at: Utc::now(),
            device_id: None,
        }
    }

    pub fn offline(user_id: Uuid) -> Self {
        Self {
            user_id,
            status: Status::Offline,
            custom_status: None,
            custom_status_expires_at: None,
            last_seen_at: Utc::now(),
            device_id: None,
        }
    }
}

impl TypingIndicator {
    pub fn new(conversation_id: Uuid, user_id: Uuid) -> Self {
        Self {
            conversation_id,
            user_id,
            started_at: Utc::now(),
        }
    }

    /// Check if typing indicator has expired (> 5 seconds)
    pub fn is_expired(&self) -> bool {
        let duration = Utc::now() - self.started_at;
        duration.num_seconds() > 5
    }
}
