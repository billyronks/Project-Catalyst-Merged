//! Agent Registry
//!
//! Pre-configured Dify agents for different use cases

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::{DifyClient, DifyError};

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(String),
    
    #[error("Dify error: {0}")]
    Dify(#[from] DifyError),
}

pub type Result<T> = std::result::Result<T, AgentError>;

/// Agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dify_app_id: String,
    pub capabilities: Vec<String>,
}

/// Agent registry
pub struct AgentRegistry {
    client: Arc<DifyClient>,
    agents: Vec<Agent>,
}

impl AgentRegistry {
    pub fn new(client: Arc<DifyClient>) -> Self {
        // Pre-configured agents
        let agents = vec![
            Agent {
                id: "customer_support".to_string(),
                name: "Customer Support Agent".to_string(),
                description: "Multi-channel intelligent customer support with FAQ knowledge".to_string(),
                dify_app_id: "support-agent-app-id".to_string(),
                capabilities: vec![
                    "answer_billing_questions".to_string(),
                    "check_account_balance".to_string(),
                    "troubleshoot_issues".to_string(),
                    "escalate_to_human".to_string(),
                ],
            },
            Agent {
                id: "aiops_analyst".to_string(),
                name: "AIOps Incident Analyst".to_string(),
                description: "Analyzes incidents, correlates metrics, suggests remediations".to_string(),
                dify_app_id: "aiops-analyst-app-id".to_string(),
                capabilities: vec![
                    "analyze_metrics".to_string(),
                    "correlate_logs".to_string(),
                    "suggest_remediation".to_string(),
                    "trigger_playbook".to_string(),
                ],
            },
            Agent {
                id: "developer_assistant".to_string(),
                name: "Developer API Assistant".to_string(),
                description: "RAG-powered help for Brivas API integration".to_string(),
                dify_app_id: "developer-assistant-app-id".to_string(),
                capabilities: vec![
                    "explain_api_endpoints".to_string(),
                    "generate_code_samples".to_string(),
                    "troubleshoot_integration".to_string(),
                    "suggest_best_practices".to_string(),
                ],
            },
            Agent {
                id: "billing_specialist".to_string(),
                name: "Billing Specialist".to_string(),
                description: "Handles billing inquiries and dispute resolution".to_string(),
                dify_app_id: "billing-specialist-app-id".to_string(),
                capabilities: vec![
                    "explain_charges".to_string(),
                    "process_refund_request".to_string(),
                    "apply_credit".to_string(),
                    "escalate_dispute".to_string(),
                ],
            },
            Agent {
                id: "campaign_advisor".to_string(),
                name: "Campaign Advisor".to_string(),
                description: "Helps design and optimize messaging campaigns".to_string(),
                dify_app_id: "campaign-advisor-app-id".to_string(),
                capabilities: vec![
                    "suggest_audience".to_string(),
                    "optimize_timing".to_string(),
                    "review_content".to_string(),
                    "estimate_performance".to_string(),
                ],
            },
        ];
        
        Self { client, agents }
    }
    
    /// List all available agents
    pub fn list(&self) -> Vec<Agent> {
        self.agents.clone()
    }
    
    /// Get agent by ID
    pub fn get(&self, id: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == id)
    }
    
    /// Chat with an agent
    pub async fn chat(
        &self,
        agent_id: &str,
        message: &str,
        conversation_id: Option<&str>,
    ) -> Result<AgentChatResponse> {
        let agent = self.get(agent_id).ok_or_else(|| AgentError::NotFound(agent_id.to_string()))?;
        
        // In production, route to actual Dify app
        // For now, return simulated response based on agent capabilities
        let response = match agent_id {
            "customer_support" => {
                format!(
                    "I understand you're asking: \"{}\". Let me help you with that. \
                    As your support agent, I can help with billing questions, account issues, \
                    or technical troubleshooting. What specific assistance do you need?",
                    message
                )
            }
            "aiops_analyst" => {
                format!(
                    "Analyzing the situation: \"{}\". \
                    I'll correlate this with recent metrics and logs. \
                    Recommended actions: 1) Check service health, 2) Review recent deployments, \
                    3) Consider triggering the recovery playbook if issues persist.",
                    message
                )
            }
            "developer_assistant" => {
                format!(
                    "Regarding your question: \"{}\". \
                    Based on the Brivas API documentation, here's what you need to know. \
                    Would you like me to provide code examples or explain the authentication flow?",
                    message
                )
            }
            _ => format!("Agent {} is processing: {}", agent_id, message),
        };
        
        Ok(AgentChatResponse {
            agent_id: agent_id.to_string(),
            message: response,
            conversation_id: conversation_id.map(|s| s.to_string())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            actions: vec![],
            suggested_replies: vec![
                "Tell me more".to_string(),
                "I need help with something else".to_string(),
                "Connect me to a human".to_string(),
            ],
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentChatResponse {
    pub agent_id: String,
    pub message: String,
    pub conversation_id: String,
    pub actions: Vec<AgentAction>,
    pub suggested_replies: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentAction {
    pub name: String,
    pub parameters: serde_json::Value,
    pub executed: bool,
}
