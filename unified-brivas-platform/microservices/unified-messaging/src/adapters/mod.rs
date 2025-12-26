//! Platform adapter trait and implementations

pub mod telegram;
pub mod whatsapp;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::model::{MessageContent, Platform, UnifiedMessage};

/// Platform message ID returned after sending
#[derive(Debug, Clone)]
pub struct PlatformMessageId(pub String);

/// Result of adapter operations
pub type AdapterResult<T> = Result<T, AdapterError>;

/// Adapter errors
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Content not supported: {0}")]
    UnsupportedContent(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Content support level
#[derive(Debug)]
pub enum ContentSupport {
    Full,
    Partial { unsupported: Vec<String> },
    None { reason: String },
}

/// Platform capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    pub max_text_length: usize,
    pub supports_buttons: bool,
    pub max_buttons: usize,
    pub supports_carousel: bool,
    pub supports_quick_replies: bool,
    pub supports_templates: bool,
    pub supports_reactions: bool,
    pub supports_read_receipts: bool,
    pub supports_typing_indicator: bool,
    pub supported_media_types: Vec<MediaType>,
    pub max_media_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
    Sticker,
}

/// Trait that all platform adapters must implement
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// Platform identifier
    fn platform(&self) -> Platform;

    /// Platform capabilities
    fn capabilities(&self) -> PlatformCapabilities;

    /// Check if content is supported
    fn supports_content(&self, content: &MessageContent) -> ContentSupport;

    /// Send a message
    async fn send_message(&self, message: &UnifiedMessage) -> AdapterResult<PlatformMessageId>;

    /// Get message status
    async fn get_message_status(&self, platform_message_id: &str) -> AdapterResult<MessageStatus>;
}

#[derive(Debug, Clone, Copy)]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
}

/// Re-export adapters
pub use telegram::TelegramAdapter;
pub use whatsapp::WhatsAppAdapter;
