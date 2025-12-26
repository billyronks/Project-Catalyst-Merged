//! GraphQL subscriptions for IM

use async_graphql::{Object, Subscription, ID, SimpleObject};
use chrono::{DateTime, Utc};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn conversation(&self, id: ID) -> Option<ConversationGql> {
        Some(ConversationGql {
            id,
            name: None,
            conversation_type: "direct".to_string(),
        })
    }

    async fn conversations(&self) -> Vec<ConversationGql> {
        vec![]
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn send_message(&self, conversation_id: ID, content: String) -> MessageGql {
        MessageGql {
            id: ID::from(uuid::Uuid::new_v4().to_string()),
            conversation_id,
            content,
            sender_id: ID::from("user-1"),
            created_at: Utc::now(),
        }
    }
}

// GraphQL Types
#[derive(SimpleObject)]
pub struct ConversationGql {
    pub id: ID,
    pub name: Option<String>,
    pub conversation_type: String,
}

#[derive(SimpleObject)]
pub struct MessageGql {
    pub id: ID,
    pub conversation_id: ID,
    pub content: String,
    pub sender_id: ID,
    pub created_at: DateTime<Utc>,
}
