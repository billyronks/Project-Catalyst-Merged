//! Messaging Protocol Types

use serde::{Deserialize, Serialize};

/// Supported messaging platforms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    WhatsApp,
    Telegram,
    FacebookMessenger,
    Instagram,
    Viber,
    Line,
    WeChat,
    Slack,
    Teams,
    Discord,
    Signal,
    Sms,
    Rcs,
    Email,
    PushNotification,
    InApp,
}

/// Message direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

/// Message content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text { body: String },
    Image { url: String, caption: Option<String> },
    Video { url: String, caption: Option<String> },
    Audio { url: String, duration_seconds: Option<u32> },
    Document { url: String, filename: String },
    Location { latitude: f64, longitude: f64, name: Option<String> },
    Contact { name: String, phone: String },
    Template { name: String, parameters: Vec<String> },
    Interactive { action: serde_json::Value },
}

/// Send message request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub request_id: String,
    pub platform: Platform,
    pub recipient: String,
    pub content: MessageContent,
    pub reply_to: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Send message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub request_id: String,
    pub message_id: String,
    pub platform_message_id: Option<String>,
    pub status: MessageStatus,
}

/// Message status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Queued,
    Sent,
    Delivered,
    Read,
    Failed,
}

/// Inbound message webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    pub message_id: String,
    pub platform: Platform,
    pub sender: String,
    pub recipient: String,
    pub content: MessageContent,
    pub timestamp: i64,
    pub platform_data: Option<serde_json::Value>,
}

/// Delivery status update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryStatusUpdate {
    pub message_id: String,
    pub status: MessageStatus,
    pub timestamp: i64,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}
