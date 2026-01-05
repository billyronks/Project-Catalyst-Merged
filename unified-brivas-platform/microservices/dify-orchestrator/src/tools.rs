//! Custom Brivas Tools for Dify
//!
//! These tools are exposed to Dify agents to interact with platform services

use serde::{Deserialize, Serialize};

/// Tool definitions for Dify
/// These are registered with Dify and can be invoked by agents

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub returns: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// Get all Brivas tool definitions for Dify
pub fn get_brivas_tools() -> Vec<ToolDefinition> {
    vec![
        // SMS Tool
        ToolDefinition {
            name: "brivas_send_sms".to_string(),
            description: "Send an SMS message to a phone number".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "to".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Recipient phone number in E.164 format".to_string(),
                },
                ToolParameter {
                    name: "message".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Message content (max 160 chars for single SMS)".to_string(),
                },
                ToolParameter {
                    name: "sender_id".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: "Sender ID to display".to_string(),
                },
            ],
            returns: "message_id: string".to_string(),
        },
        
        // Voice Call Tool
        ToolDefinition {
            name: "brivas_initiate_call".to_string(),
            description: "Initiate a voice call to a phone number".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "to".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Recipient phone number".to_string(),
                },
                ToolParameter {
                    name: "from".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Caller ID".to_string(),
                },
                ToolParameter {
                    name: "tts_message".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: "Text-to-speech message to play".to_string(),
                },
            ],
            returns: "call_id: string".to_string(),
        },
        
        // USSD Tool
        ToolDefinition {
            name: "brivas_send_ussd".to_string(),
            description: "Send USSD menu or response".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "session_id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "USSD session ID".to_string(),
                },
                ToolParameter {
                    name: "message".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "USSD menu or response text".to_string(),
                },
                ToolParameter {
                    name: "end_session".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    description: "Whether to end the USSD session".to_string(),
                },
            ],
            returns: "success: boolean".to_string(),
        },
        
        // Billing Tools
        ToolDefinition {
            name: "brivas_check_balance".to_string(),
            description: "Check account balance for a user".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "account_id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Account ID to check".to_string(),
                },
            ],
            returns: "balance: number, currency: string".to_string(),
        },
        
        ToolDefinition {
            name: "brivas_get_transactions".to_string(),
            description: "Get recent transactions for an account".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "account_id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Account ID".to_string(),
                },
                ToolParameter {
                    name: "limit".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                    description: "Number of transactions to retrieve".to_string(),
                },
            ],
            returns: "transactions: array".to_string(),
        },
        
        ToolDefinition {
            name: "brivas_apply_credit".to_string(),
            description: "Apply a credit to an account (requires approval for > $50)".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "account_id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Account ID".to_string(),
                },
                ToolParameter {
                    name: "amount".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    description: "Credit amount".to_string(),
                },
                ToolParameter {
                    name: "reason".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Reason for credit".to_string(),
                },
            ],
            returns: "credit_id: string, approved: boolean".to_string(),
        },
        
        // Campaign Tools
        ToolDefinition {
            name: "brivas_create_campaign".to_string(),
            description: "Create a new messaging campaign".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Campaign name".to_string(),
                },
                ToolParameter {
                    name: "message".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Message template".to_string(),
                },
                ToolParameter {
                    name: "audience_filter".to_string(),
                    param_type: "object".to_string(),
                    required: false,
                    description: "Audience filter criteria".to_string(),
                },
                ToolParameter {
                    name: "schedule".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: "ISO 8601 schedule date/time".to_string(),
                },
            ],
            returns: "campaign_id: string, status: string".to_string(),
        },
        
        // AIOps Tools
        ToolDefinition {
            name: "brivas_get_service_health".to_string(),
            description: "Get health status of platform services".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "service".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: "Specific service name or 'all'".to_string(),
                },
            ],
            returns: "services: array with name, status, uptime".to_string(),
        },
        
        ToolDefinition {
            name: "brivas_trigger_playbook".to_string(),
            description: "Trigger an AIOps remediation playbook".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "playbook_id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Playbook ID (e.g., 'smpp_recovery', 'service_restart')".to_string(),
                },
                ToolParameter {
                    name: "parameters".to_string(),
                    param_type: "object".to_string(),
                    required: false,
                    description: "Playbook parameters".to_string(),
                },
            ],
            returns: "execution_id: string, status: string".to_string(),
        },
        
        // Knowledge Tools
        ToolDefinition {
            name: "brivas_search_docs".to_string(),
            description: "Search platform documentation and FAQs".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "query".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Search query".to_string(),
                },
                ToolParameter {
                    name: "category".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: "Category: api, billing, troubleshooting, general".to_string(),
                },
            ],
            returns: "results: array with title, content, score".to_string(),
        },
    ]
}

/// Execute a tool request from Dify
pub async fn execute_tool(tool_name: &str, parameters: serde_json::Value) -> Result<serde_json::Value, String> {
    match tool_name {
        "brivas_send_sms" => {
            // In production, call actual SMS service
            let to = parameters.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let message = parameters.get("message").and_then(|v| v.as_str()).unwrap_or("");
            
            Ok(serde_json::json!({
                "message_id": format!("msg_{}", uuid::Uuid::new_v4()),
                "status": "sent",
                "to": to,
                "message_preview": message.chars().take(50).collect::<String>()
            }))
        }
        "brivas_check_balance" => {
            Ok(serde_json::json!({
                "balance": 1500.50,
                "currency": "USD",
                "last_updated": chrono::Utc::now().to_rfc3339()
            }))
        }
        "brivas_get_service_health" => {
            Ok(serde_json::json!({
                "services": [
                    {"name": "smsc", "status": "healthy", "uptime": "99.99%"},
                    {"name": "billing", "status": "healthy", "uptime": "99.95%"},
                    {"name": "ussd-gateway", "status": "healthy", "uptime": "99.99%"}
                ],
                "overall": "healthy"
            }))
        }
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}
