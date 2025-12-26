//! Voice/IVR Protocol Types

use serde::{Deserialize, Serialize};

/// Flash Call Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashCallRequest {
    pub request_id: String,
    pub destination: String,
    pub cli_prefix: String,
    #[serde(default = "default_otp_length")]
    pub otp_length: u8,
    pub callback_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

fn default_otp_length() -> u8 { 4 }

/// Flash Call Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashCallResponse {
    pub request_id: String,
    pub call_id: String,
    pub otp: String,
    pub status: FlashCallStatus,
}

/// Flash Call Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlashCallStatus {
    Initiated,
    Ringing,
    Completed,
    Failed,
}

/// OTP Verification Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub request_id: String,
    pub otp: String,
}

/// OTP Verification Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    pub request_id: String,
    pub verified: bool,
    pub reason: Option<String>,
}

/// IVR Flow Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrFlow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub entry_node: String,
    pub nodes: Vec<IvrNode>,
}

/// IVR Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrNode {
    pub id: String,
    pub node_type: String,
    pub config: serde_json::Value,
    pub transitions: Vec<IvrTransition>,
}

/// IVR Transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrTransition {
    pub condition: String,
    pub next_node: String,
}

/// Campaign Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub message_type: String,
    pub audio_url: Option<String>,
    pub tts_text: Option<String>,
    pub tts_language: Option<String>,
    pub recipients: Vec<String>,
    pub caller_id: String,
    pub scheduled_at: Option<i64>,
    pub throttle_cps: Option<u32>,
    pub retry_attempts: Option<u32>,
}

/// Campaign Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub total_recipients: usize,
    pub completed: usize,
    pub failed: usize,
}

/// Dialer Session Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartDialerSessionRequest {
    pub campaign_id: String,
    pub agents: Vec<String>,
}

/// Dialer Session Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialerSessionResponse {
    pub session_id: String,
    pub campaign_id: String,
    pub status: String,
    pub calls_placed: u64,
    pub calls_connected: u64,
}

/// Voice Chat Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendVoiceChatRequest {
    pub platform: String,
    pub recipient: String,
    pub audio_base64: String,
    pub audio_format: String,
}

/// Voice Chat Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendVoiceChatResponse {
    pub message_id: String,
    pub status: String,
}
