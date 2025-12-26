//! Hasura-style Actions

use serde::{Deserialize, Serialize};

/// Action definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub name: String,
    pub kind: ActionKind,
    pub handler: ActionHandler,
    pub input_type: String,
    pub output_type: String,
    pub permissions: Vec<String>,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionKind {
    Query,
    Mutation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionHandler {
    InternalService { service: String, method: String },
    Webhook { url: String },
}

/// Built-in Brivas actions
pub fn get_brivas_actions() -> Vec<ActionDefinition> {
    vec![
        ActionDefinition {
            name: "sendInstantMessage".to_string(),
            kind: ActionKind::Mutation,
            handler: ActionHandler::InternalService {
                service: "instant-messaging".to_string(),
                method: "sendMessage".to_string(),
            },
            input_type: "SendImInput".to_string(),
            output_type: "Message".to_string(),
            permissions: vec!["user".to_string(), "admin".to_string()],
            timeout_seconds: 30,
        },
        ActionDefinition {
            name: "sendRichCard".to_string(),
            kind: ActionKind::Mutation,
            handler: ActionHandler::InternalService {
                service: "rcs-messaging".to_string(),
                method: "sendRichCard".to_string(),
            },
            input_type: "RichCardInput".to_string(),
            output_type: "RcsMessage".to_string(),
            permissions: vec!["agent".to_string(), "admin".to_string()],
            timeout_seconds: 30,
        },
        ActionDefinition {
            name: "initiateVoiceCall".to_string(),
            kind: ActionKind::Mutation,
            handler: ActionHandler::InternalService {
                service: "voice-ivr".to_string(),
                method: "initiateCall".to_string(),
            },
            input_type: "InitiateCallInput".to_string(),
            output_type: "CallSession".to_string(),
            permissions: vec!["campaign".to_string(), "admin".to_string()],
            timeout_seconds: 60,
        },
        ActionDefinition {
            name: "queryAnalytics".to_string(),
            kind: ActionKind::Query,
            handler: ActionHandler::InternalService {
                service: "analytics-service".to_string(),
                method: "query".to_string(),
            },
            input_type: "AnalyticsQuery".to_string(),
            output_type: "AnalyticsResult".to_string(),
            permissions: vec!["analyst".to_string(), "admin".to_string()],
            timeout_seconds: 60,
        },
    ]
}
