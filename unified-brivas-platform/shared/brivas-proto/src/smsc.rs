//! SMSC Protocol Types

use serde::{Deserialize, Serialize};

/// SMS Send Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSmsRequest {
    pub request_id: String,
    pub source: String,
    pub destination: String,
    pub message: String,
    pub message_type: SmsMessageType,
    pub validity_period: Option<u32>,
    pub priority: Option<SmsPriority>,
    pub callback_url: Option<String>,
}

/// SMS Message Type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SmsMessageType {
    Normal,
    Flash,
    Binary,
    Unicode,
}

/// SMS Priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SmsPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// SMS Send Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSmsResponse {
    pub request_id: String,
    pub message_id: String,
    pub segments: u32,
    pub status: SmsStatus,
}

/// SMS Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SmsStatus {
    Accepted,
    Queued,
    Sent,
    Delivered,
    Failed,
    Rejected,
    Expired,
}

/// Delivery Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReport {
    pub message_id: String,
    pub status: SmsStatus,
    pub delivered_at: Option<i64>,
    pub error_code: Option<u32>,
    pub mcc_mnc: Option<String>,
}

/// SMPP Session Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmppSessionInfo {
    pub session_id: String,
    pub system_id: String,
    pub bind_type: SmppBindType,
    pub connected_at: i64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

/// SMPP Bind Type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SmppBindType {
    Transmitter,
    Receiver,
    Transceiver,
}

/// Routing Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub id: String,
    pub name: String,
    pub priority: u32,
    pub prefix_match: Option<String>,
    pub mcc_mnc_match: Option<String>,
    pub gateway_id: String,
    pub fallback_gateway_id: Option<String>,
}
