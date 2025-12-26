//! Presence Manager - Real-time presence tracking

use dashmap::DashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::{PresenceStatus, Status, TypingIndicator};

/// In-memory presence manager
pub struct PresenceManager {
    presence: DashMap<Uuid, PresenceStatus>,
    typing: DashMap<(Uuid, Uuid), TypingIndicator>, // (conversation_id, user_id)
}

impl PresenceManager {
    pub fn new() -> Self {
        Self {
            presence: DashMap::new(),
            typing: DashMap::new(),
        }
    }

    /// Update user presence
    pub fn update_presence(&self, user_id: Uuid, status: Status) {
        self.presence.insert(user_id, PresenceStatus {
            user_id,
            status,
            custom_status: None,
            custom_status_expires_at: None,
            last_seen_at: Utc::now(),
            device_id: None,
        });
    }

    /// Get user presence
    pub fn get_presence(&self, user_id: &Uuid) -> Option<PresenceStatus> {
        self.presence.get(user_id).map(|p| p.clone())
    }

    /// Get presence for multiple users
    pub fn get_bulk_presence(&self, user_ids: &[Uuid]) -> Vec<PresenceStatus> {
        user_ids
            .iter()
            .filter_map(|id| self.presence.get(id).map(|p| p.clone()))
            .collect()
    }

    /// Set user as offline
    pub fn set_offline(&self, user_id: &Uuid) {
        if let Some(mut presence) = self.presence.get_mut(user_id) {
            presence.status = Status::Offline;
            presence.last_seen_at = Utc::now();
        }
    }

    /// Update typing indicator
    pub fn set_typing(&self, conversation_id: Uuid, user_id: Uuid, is_typing: bool) {
        let key = (conversation_id, user_id);
        if is_typing {
            self.typing.insert(key, TypingIndicator::new(conversation_id, user_id));
        } else {
            self.typing.remove(&key);
        }
    }

    /// Get typing indicators for a conversation
    pub fn get_typing(&self, conversation_id: &Uuid) -> Vec<Uuid> {
        self.typing
            .iter()
            .filter(|entry| &entry.key().0 == conversation_id && !entry.value().is_expired())
            .map(|entry| entry.key().1)
            .collect()
    }

    /// Clean up expired typing indicators
    pub fn cleanup_expired_typing(&self) {
        self.typing.retain(|_, indicator| !indicator.is_expired());
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}
