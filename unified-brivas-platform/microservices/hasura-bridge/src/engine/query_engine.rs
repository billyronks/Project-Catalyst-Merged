//! GraphQL Query Engine

use async_graphql::{EmptySubscription, Schema};
use brivas_core::Result;

use crate::config::HasuraConfig;
use crate::schema::unified_schema::{QueryRoot, MutationRoot};

pub type HasuraSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Hasura-compatible GraphQL engine with LumaDB backend
pub struct HasuraEngine {
    config: HasuraConfig,
}

impl HasuraEngine {
    pub async fn new(config: &HasuraConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Build the GraphQL schema
    pub async fn build_schema(&self) -> Result<HasuraSchema> {
        let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
            .data(self.config.clone())
            .finish();

        Ok(schema)
    }
}
