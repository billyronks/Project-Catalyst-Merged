//! Viber adapter

use async_trait::async_trait;
use serde_json::json;

use super::{
    AdapterError, AdapterResult, ContentSupport, MediaType, MessageStatus, PlatformAdapter,
    PlatformCapabilities, PlatformMessageId,
};
use crate::model::{MessageContent, Platform, UnifiedMessage};

pub struct ViberAdapter {
    auth_token: String,
    sender_name: String,
    http_client: reqwest::Client,
}

impl ViberAdapter {
    pub fn new(auth_token: String, sender_name: String) -> Self {
        Self { auth_token, sender_name, http_client: reqwest::Client::new() }
    }
}

#[async_trait]
impl PlatformAdapter for ViberAdapter {
    fn platform(&self) -> Platform { Platform::Viber }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            max_text_length: 7000,
            supports_buttons: true,
            max_buttons: 6,
            supports_carousel: true,
            supports_quick_replies: true,
            supports_templates: false,
            supports_reactions: false,
            supports_read_receipts: true,
            supports_typing_indicator: false,
            supported_media_types: vec![MediaType::Image, MediaType::Video, MediaType::Document],
            max_media_size_bytes: 200 * 1024 * 1024,
        }
    }

    fn supports_content(&self, _: &MessageContent) -> ContentSupport { ContentSupport::Full }

    async fn send_message(&self, message: &UnifiedMessage) -> AdapterResult<PlatformMessageId> {
        let payload = match &message.content {
            MessageContent::Text { body } => json!({
                "receiver": message.recipient_id,
                "type": "text",
                "sender": { "name": self.sender_name },
                "text": body
            }),
            MessageContent::Image { url, .. } => json!({
                "receiver": message.recipient_id,
                "type": "picture",
                "sender": { "name": self.sender_name },
                "media": url
            }),
            _ => json!({
                "receiver": message.recipient_id,
                "type": "text",
                "sender": { "name": self.sender_name },
                "text": "Unsupported"
            }),
        };

        let response = self.http_client
            .post("https://chatapi.viber.com/pa/send_message")
            .header("X-Viber-Auth-Token", &self.auth_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AdapterError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await.map_err(|e| AdapterError::Parse(e.to_string()))?;
        Ok(PlatformMessageId(result["message_token"].to_string()))
    }

    async fn get_message_status(&self, _: &str) -> AdapterResult<MessageStatus> { Ok(MessageStatus::Sent) }
}
