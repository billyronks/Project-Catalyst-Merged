//! Dify Orchestrator Configuration

use brivas_core::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DifyConfig {
    pub http_bind: String,
    pub lumadb_url: String,
    pub dify_base_url: String,
    pub dify_api_key: String,
    pub default_agent_id: String,
    pub campaign_workflow_id: Option<String>,
    pub support_agent_id: Option<String>,
    pub aiops_agent_id: Option<String>,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dify_app_id: String,
    pub system_prompt_override: Option<String>,
}

impl DifyConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_bind: std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            dify_base_url: std::env::var("DIFY_BASE_URL")
                .unwrap_or_else(|_| "https://api.dify.ai/v1".to_string()),
            dify_api_key: std::env::var("DIFY_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            default_agent_id: std::env::var("DIFY_DEFAULT_AGENT")
                .unwrap_or_else(|_| "general_assistant".to_string()),
            campaign_workflow_id: std::env::var("DIFY_CAMPAIGN_WORKFLOW").ok(),
            support_agent_id: std::env::var("DIFY_SUPPORT_AGENT").ok(),
            aiops_agent_id: std::env::var("DIFY_AIOPS_AGENT").ok(),
            request_timeout_secs: std::env::var("DIFY_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        })
    }
}
