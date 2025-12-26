//! RCS Message Store - LumaDB persistence

use brivas_core::Result;
use dashmap::DashMap;
use uuid::Uuid;

use crate::domain::RcsMessage;

pub struct RcsMessageStore {
    db_url: String,
    cache: DashMap<Uuid, RcsMessage>,
}

impl RcsMessageStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        Ok(Self {
            db_url: db_url.to_string(),
            cache: DashMap::new(),
        })
    }

    pub async fn store(&self, message: RcsMessage) -> Result<Uuid> {
        let id = message.id;
        self.cache.insert(id, message);
        // TODO: Persist to LumaDB
        Ok(id)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<RcsMessage>> {
        Ok(self.cache.get(id).map(|m| m.clone()))
    }

    pub async fn update_status(&self, id: &Uuid, status: brivas_rcs_sdk::message::RcsMessageStatus) -> Result<()> {
        if let Some(mut msg) = self.cache.get_mut(id) {
            msg.status = status;
        }
        Ok(())
    }
}
