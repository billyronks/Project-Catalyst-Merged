//! RCS Agent/Brand types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// RCS Agent (Brand) entity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsAgent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub logo_url: String,
    pub hero_image_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: Option<String>,
    pub category: AgentCategory,
    pub webhook_url: String,
    pub verification_status: AgentVerificationStatus,
    pub capabilities: AgentCapabilities,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentCategory {
    Transactional,
    Promotional,
    Conversational,
    Support,
}

/// Agent verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentVerificationStatus {
    Pending,
    Verified,
    Rejected,
    Suspended,
}

/// Agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    pub rich_card: bool,
    pub carousel: bool,
    pub file_transfer: bool,
    pub location_sharing: bool,
    pub suggested_replies: bool,
    pub suggested_actions: bool,
}

impl RcsAgent {
    /// Create a new RCS agent
    pub fn new(
        tenant_id: Uuid,
        name: String,
        display_name: String,
        logo_url: String,
        primary_color: String,
        webhook_url: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name,
            display_name,
            description: None,
            logo_url,
            hero_image_url: None,
            primary_color,
            secondary_color: None,
            category: AgentCategory::Transactional,
            webhook_url,
            verification_status: AgentVerificationStatus::Pending,
            capabilities: AgentCapabilities::default(),
            created_at: now,
            updated_at: now,
        }
    }
}
