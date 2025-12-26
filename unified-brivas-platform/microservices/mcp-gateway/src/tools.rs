//! Brivas MCP Tools

use async_trait::async_trait;
use brivas_mcp_sdk::tool::{Tool, ToolResult, ToolError, ToolDefinition};
use serde_json::{json, Value};

/// Collection of Brivas MCP tools
pub struct BrivasTools {
    tools: Vec<Box<dyn Tool>>,
}

impl BrivasTools {
    pub fn new() -> Self {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(SendSmsTool),
            Box::new(SendRcsTool),
            Box::new(CreateCampaignTool),
            Box::new(GetConversationTool),
            Box::new(QueryAnalyticsTool),
            Box::new(InitiateCallTool),
            Box::new(CreateUssdMenuTool),
        ];
        Self { tools }
    }

    pub fn list(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|t| ToolDefinition::from(t.as_ref())).collect()
    }

    pub async fn execute(&self, name: &str, args: Value) -> Result<ToolResult, ToolError> {
        for tool in &self.tools {
            if tool.name() == name {
                return tool.execute(args).await;
            }
        }
        Err(ToolError::NotFound(name.to_string()))
    }
}

// Tool implementations

struct SendSmsTool;

#[async_trait]
impl Tool for SendSmsTool {
    fn name(&self) -> &str { "brivas_send_sms" }
    fn description(&self) -> &str { "Send an SMS message to a phone number" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "to": { "type": "string", "description": "Recipient phone number in E.164 format" },
                "message": { "type": "string", "description": "SMS message text (max 160 chars for single, 1600 for multipart)" },
                "sender_id": { "type": "string", "description": "Sender ID or short code" }
            },
            "required": ["to", "message"]
        })
    }
    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let to = args.get("to").and_then(|v| v.as_str()).ok_or(ToolError::InvalidInput("to required".into()))?;
        let message = args.get("message").and_then(|v| v.as_str()).ok_or(ToolError::InvalidInput("message required".into()))?;
        
        // TODO: Call SMSC service
        Ok(ToolResult::text(format!("SMS sent to {} with message: {}", to, message)))
    }
}

struct SendRcsTool;

#[async_trait]
impl Tool for SendRcsTool {
    fn name(&self) -> &str { "brivas_send_rcs" }
    fn description(&self) -> &str { "Send an RCS rich card or carousel message" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "RCS Agent ID" },
                "to": { "type": "string", "description": "Recipient phone number" },
                "message_type": { "type": "string", "enum": ["text", "rich_card", "carousel"] },
                "content": { "type": "object", "description": "Message content" }
            },
            "required": ["agent_id", "to", "message_type", "content"]
        })
    }
    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let to = args.get("to").and_then(|v| v.as_str()).unwrap_or("");
        let msg_type = args.get("message_type").and_then(|v| v.as_str()).unwrap_or("text");
        
        Ok(ToolResult::text(format!("RCS {} sent to {}", msg_type, to)))
    }
}

struct CreateCampaignTool;

#[async_trait]
impl Tool for CreateCampaignTool {
    fn name(&self) -> &str { "brivas_create_campaign" }
    fn description(&self) -> &str { "Create a new messaging campaign" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "channel": { "type": "string", "enum": ["sms", "rcs", "voice", "ussd"] },
                "audience_filter": { "type": "object" },
                "message_template": { "type": "string" },
                "schedule": { "type": "string", "description": "ISO8601 datetime or 'now'" }
            },
            "required": ["name", "channel", "message_template"]
        })
    }
    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("Untitled");
        let channel = args.get("channel").and_then(|v| v.as_str()).unwrap_or("sms");
        
        Ok(ToolResult::text(format!("Campaign '{}' created for {} channel", name, channel)))
    }
}

struct GetConversationTool;

#[async_trait]
impl Tool for GetConversationTool {
    fn name(&self) -> &str { "brivas_get_conversation" }
    fn description(&self) -> &str { "Get conversation history with a contact" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "phone_number": { "type": "string" },
                "limit": { "type": "integer", "default": 20 }
            },
            "required": ["phone_number"]
        })
    }
    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let phone = args.get("phone_number").and_then(|v| v.as_str()).unwrap_or("");
        
        Ok(ToolResult::text(format!("Conversation history for {}: [No messages found]", phone)))
    }
}

struct QueryAnalyticsTool;

#[async_trait]
impl Tool for QueryAnalyticsTool {
    fn name(&self) -> &str { "brivas_query_analytics" }
    fn description(&self) -> &str { "Query messaging analytics and metrics" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "metric": { "type": "string", "enum": ["messages_sent", "delivery_rate", "response_rate", "cost"] },
                "channel": { "type": "string", "enum": ["all", "sms", "rcs", "voice", "ussd"] },
                "period": { "type": "string", "enum": ["today", "7d", "30d", "custom"] },
                "from": { "type": "string" },
                "to": { "type": "string" }
            },
            "required": ["metric"]
        })
    }
    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let metric = args.get("metric").and_then(|v| v.as_str()).unwrap_or("");
        
        Ok(ToolResult::text(format!("Analytics for {}: 0", metric)))
    }
}

struct InitiateCallTool;

#[async_trait]
impl Tool for InitiateCallTool {
    fn name(&self) -> &str { "brivas_initiate_call" }
    fn description(&self) -> &str { "Initiate an outbound voice call" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "from": { "type": "string" },
                "to": { "type": "string" },
                "ivr_flow_id": { "type": "string" }
            },
            "required": ["from", "to"]
        })
    }
    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let to = args.get("to").and_then(|v| v.as_str()).unwrap_or("");
        
        Ok(ToolResult::text(format!("Call initiated to {}", to)))
    }
}

struct CreateUssdMenuTool;

#[async_trait]
impl Tool for CreateUssdMenuTool {
    fn name(&self) -> &str { "brivas_create_ussd_menu" }
    fn description(&self) -> &str { "Create a USSD menu flow" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "short_code": { "type": "string" },
                "menu_structure": { "type": "object" }
            },
            "required": ["short_code", "menu_structure"]
        })
    }
    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError> {
        let code = args.get("short_code").and_then(|v| v.as_str()).unwrap_or("");
        
        Ok(ToolResult::text(format!("USSD menu created for {}", code)))
    }
}
