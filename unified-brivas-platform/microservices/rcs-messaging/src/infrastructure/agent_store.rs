//! Agent Store - LumaDB persistence

use brivas_core::Result;
use dashmap::DashMap;
use uuid::Uuid;

use crate::domain::RcsAgent;

pub struct AgentStore {
    db_url: String,
    cache: DashMap<Uuid, RcsAgent>,
}

impl AgentStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        Ok(Self {
            db_url: db_url.to_string(),
            cache: DashMap::new(),
        })
    }

    pub async fn create(&self, agent: RcsAgent) -> Result<Uuid> {
        let id = agent.id;
        self.cache.insert(id, agent);
        // TODO: Persist to LumaDB
        Ok(id)
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<RcsAgent>> {
        Ok(self.cache.get(id).map(|a| a.clone()))
    }

    pub async fn list_by_tenant(&self, tenant_id: &Uuid) -> Result<Vec<RcsAgent>> {
        let agents: Vec<RcsAgent> = self.cache
            .iter()
            .filter(|e| &e.value().tenant_id == tenant_id)
            .map(|e| e.value().clone())
            .collect();
        Ok(agents)
    }

    pub async fn delete(&self, id: &Uuid) -> Result<()> {
        self.cache.remove(id);
        Ok(())
    }
}
