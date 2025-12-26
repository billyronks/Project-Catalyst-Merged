//! Message Store - LumaDB persistence

use brivas_core::Result;
use dashmap::DashMap;
use uuid::Uuid;
use std::collections::VecDeque;

use crate::domain::Message;

/// LumaDB-backed message store
pub struct MessageStore {
    db_url: String,
    // In-memory cache: conversation_id -> messages (most recent first)
    cache: DashMap<Uuid, VecDeque<Message>>,
    max_cache_size: usize,
}

impl MessageStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        Ok(Self {
            db_url: db_url.to_string(),
            cache: DashMap::new(),
            max_cache_size: 1000,
        })
    }

    /// Store a message
    pub async fn store(&self, message: Message) -> Result<Uuid> {
        let id = message.id;
        let conversation_id = message.conversation_id;
        
        self.cache
            .entry(conversation_id)
            .or_insert_with(VecDeque::new)
            .push_front(message);
        
        // Trim cache if too large
        if let Some(mut messages) = self.cache.get_mut(&conversation_id) {
            while messages.len() > self.max_cache_size {
                messages.pop_back();
            }
        }
        
        // TODO: Persist to LumaDB
        Ok(id)
    }

    /// Get messages for a conversation
    pub async fn get_messages(
        &self,
        conversation_id: &Uuid,
        limit: usize,
        before: Option<Uuid>,
    ) -> Result<Vec<Message>> {
        if let Some(messages) = self.cache.get(conversation_id) {
            let iter = messages.iter();
            
            let filtered: Vec<Message> = if let Some(before_id) = before {
                iter.skip_while(|m| m.id != before_id)
                    .skip(1)
                    .take(limit)
                    .cloned()
                    .collect()
            } else {
                iter.take(limit).cloned().collect()
            };
            
            return Ok(filtered);
        }
        
        // TODO: Load from LumaDB
        Ok(vec![])
    }

    /// Get message by ID
    pub async fn get(&self, conversation_id: &Uuid, message_id: &Uuid) -> Result<Option<Message>> {
        if let Some(messages) = self.cache.get(conversation_id) {
            if let Some(msg) = messages.iter().find(|m| &m.id == message_id) {
                return Ok(Some(msg.clone()));
            }
        }
        Ok(None)
    }

    /// Delete message
    pub async fn delete(&self, conversation_id: &Uuid, message_id: &Uuid) -> Result<()> {
        if let Some(mut messages) = self.cache.get_mut(conversation_id) {
            messages.retain(|m| &m.id != message_id);
        }
        // TODO: Delete from LumaDB
        Ok(())
    }
}
