//! Conversation Store - LumaDB persistence

use brivas_core::Result;
use dashmap::DashMap;
use uuid::Uuid;

use crate::domain::Conversation;

/// LumaDB-backed conversation store
pub struct ConversationStore {
    db_url: String,
    // In-memory cache for hot data
    cache: DashMap<Uuid, Conversation>,
}

impl ConversationStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        Ok(Self {
            db_url: db_url.to_string(),
            cache: DashMap::new(),
        })
    }

    /// Create a new conversation
    pub async fn create(&self, conversation: Conversation) -> Result<Uuid> {
        let id = conversation.id;
        self.cache.insert(id, conversation);
        // TODO: Persist to LumaDB
        Ok(id)
    }

    /// Get conversation by ID
    pub async fn get(&self, id: &Uuid) -> Result<Option<Conversation>> {
        if let Some(conv) = self.cache.get(id) {
            return Ok(Some(conv.clone()));
        }
        // TODO: Load from LumaDB
        Ok(None)
    }

    /// List conversations for a user
    pub async fn list_for_user(&self, user_id: &Uuid) -> Result<Vec<Conversation>> {
        let conversations: Vec<Conversation> = self.cache
            .iter()
            .filter(|entry| {
                entry.value().participants.iter().any(|p| &p.user_id == user_id)
            })
            .map(|entry| entry.value().clone())
            .collect();
        Ok(conversations)
    }

    /// Delete conversation
    pub async fn delete(&self, id: &Uuid) -> Result<()> {
        self.cache.remove(id);
        // TODO: Delete from LumaDB
        Ok(())
    }
}
