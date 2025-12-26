//! Core domain types used across all microservices

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique message identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);

impl MessageId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Account identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub String);

impl AccountId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Conversation identifier for messaging
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationId(pub String);

/// Session identifier for USSD
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Message status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Pending,
    Queued,
    Sent,
    Delivered,
    Read,
    Failed,
    Expired,
    Rejected,
}

/// Message direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

/// Telecom operator/carrier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operator {
    Mtn,
    Airtel,
    Glo,
    NineMobile,
    Unknown,
}

impl Operator {
    /// Detect operator from Nigerian phone number prefix
    pub fn from_msisdn(msisdn: &str) -> Self {
        let normalized = msisdn.trim_start_matches('+').trim_start_matches("234");
        let prefix = if normalized.starts_with('0') {
            &normalized[..4]
        } else {
            &format!("0{}", &normalized[..3])
        };

        match prefix {
            "0803" | "0806" | "0703" | "0706" | "0813" | "0816" | "0810" | "0814" | "0903"
            | "0906" | "0913" | "0916" => Self::Mtn,
            "0805" | "0807" | "0705" | "0815" | "0811" | "0905" | "0915" => Self::Glo,
            "0802" | "0808" | "0708" | "0812" | "0701" | "0902" | "0901" | "0907" | "0912" => {
                Self::Airtel
            }
            "0809" | "0817" | "0818" | "0908" | "0909" => Self::NineMobile,
            _ => Self::Unknown,
        }
    }
}

/// Phone number with formatting utilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn new(number: impl Into<String>) -> Self {
        Self(Self::normalize(number.into()))
    }

    fn normalize(number: String) -> String {
        let cleaned: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        if cleaned.starts_with("234") {
            cleaned
        } else if cleaned.starts_with('0') {
            format!("234{}", &cleaned[1..])
        } else {
            format!("234{}", cleaned)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn operator(&self) -> Operator {
        Operator::from_msisdn(&self.0)
    }

    pub fn international(&self) -> String {
        format!("+{}", self.0)
    }

    pub fn local(&self) -> String {
        format!("0{}", &self.0[3..])
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Sender ID for messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderId(pub String);

/// Request context for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub trace_id: String,
    pub span_id: String,
    pub account_id: Option<AccountId>,
    pub tenant_id: Option<String>,
    pub request_id: String,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string()[..16].to_string(),
            account_id: None,
            tenant_id: None,
            request_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn with_account(mut self, account_id: AccountId) -> Self {
        self.account_id = Some(account_id);
        self
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Timestamp wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp(pub DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}
