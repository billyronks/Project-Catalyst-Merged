//! Brivas MCP Resources

use brivas_mcp_sdk::resource::{Resource, ResourceContent};

/// Collection of Brivas MCP resources
pub struct BrivasResources {
    resources: Vec<Resource>,
}

impl BrivasResources {
    pub fn new() -> Self {
        Self {
            resources: vec![
                Resource::new("brivas://analytics/summary", "Analytics Summary")
                    .with_description("Real-time messaging analytics summary")
                    .with_mime_type("application/json"),
                    
                Resource::new("brivas://campaigns/active", "Active Campaigns")
                    .with_description("List of currently active campaigns")
                    .with_mime_type("application/json"),
                    
                Resource::new("brivas://contacts/recent", "Recent Contacts")
                    .with_description("Recently contacted phone numbers")
                    .with_mime_type("application/json"),
                    
                Resource::new("brivas://credits/balance", "Credit Balance")
                    .with_description("Current account credit balance")
                    .with_mime_type("application/json"),
                    
                Resource::new("brivas://agents/list", "RCS Agents")
                    .with_description("List of registered RCS agents")
                    .with_mime_type("application/json"),
                    
                Resource::new("brivas://conversations/{phone}", "Conversation History")
                    .with_description("Message history for a specific contact")
                    .with_mime_type("application/json"),
            ],
        }
    }

    pub fn list(&self) -> &[Resource] {
        &self.resources
    }

    pub async fn read(&self, uri: &str) -> Option<ResourceContent> {
        match uri {
            "brivas://analytics/summary" => {
                Some(ResourceContent::json(uri, r#"{
                    "messages_sent_today": 12500,
                    "delivery_rate": 0.987,
                    "response_rate": 0.23,
                    "active_campaigns": 5
                }"#))
            }
            "brivas://campaigns/active" => {
                Some(ResourceContent::json(uri, "[]"))
            }
            "brivas://contacts/recent" => {
                Some(ResourceContent::json(uri, "[]"))
            }
            "brivas://credits/balance" => {
                Some(ResourceContent::json(uri, r#"{"balance": 50000, "currency": "NGN"}"#))
            }
            "brivas://agents/list" => {
                Some(ResourceContent::json(uri, "[]"))
            }
            _ if uri.starts_with("brivas://conversations/") => {
                Some(ResourceContent::json(uri, "[]"))
            }
            _ => None,
        }
    }
}

impl Default for BrivasResources {
    fn default() -> Self {
        Self::new()
    }
}
