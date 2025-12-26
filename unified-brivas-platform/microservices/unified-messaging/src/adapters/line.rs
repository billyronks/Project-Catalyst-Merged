//! LINE Messaging API adapter

use async_trait::async_trait;
use serde_json::json;

use super::{
    AdapterError, AdapterResult, ContentSupport, MediaType, MessageStatus, PlatformAdapter,
    PlatformCapabilities, PlatformMessageId,
};
use crate::model::{MessageContent, Platform, UnifiedMessage};

pub struct LineAdapter {
    channel_access_token: String,
    http_client: reqwest::Client,
}

impl LineAdapter {
    pub fn new(channel_access_token: String) -> Self {
        Self { channel_access_token, http_client: reqwest::Client::new() }
    }
}

#[async_trait]
impl PlatformAdapter for LineAdapter {
    fn platform(&self) -> Platform { Platform::Line }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            max_text_length: 5000,
            supports_buttons: true,
            max_buttons: 13,
            supports_carousel: true,
            supports_quick_replies: true,
            supports_templates: true,
            supports_reactions: false,
            supports_read_receipts: false,
            supports_typing_indicator: false,
            supported_media_types: vec![MediaType::Image, MediaType::Video, MediaType::Audio, MediaType::Sticker],
            max_media_size_bytes: 300 * 1024 * 1024,
        }
    }

    fn supports_content(&self, _: &MessageContent) -> ContentSupport { ContentSupport::Full }

    async fn send_message(&self, message: &UnifiedMessage) -> AdapterResult<PlatformMessageId> {
        let msg = match &message.content {
            MessageContent::Text { body } => json!({ "type": "text", "text": body }),
            MessageContent::Image { url, .. } => json!({ "type": "image", "originalContentUrl": url, "previewImageUrl": url }),
            MessageContent::Interactive { body, action, .. } => {
                let buttons: Vec<_> = action.buttons.as_ref().map(|b| b.iter().map(|btn| json!({
                    "type": "message", "label": btn.title, "text": btn.id
                })).collect()).unwrap_or_default();
                json!({
                    "type": "template",
                    "altText": body,
                    "template": { "type": "buttons", "text": body, "actions": buttons }
                })
            }
            _ => json!({ "type": "text", "text": "Unsupported" }),
        };

        let response = self.http_client
            .post("https://api.line.me/v2/bot/message/push")
            .bearer_auth(&self.channel_access_token)
            .json(&json!({ "to": message.recipient_id, "messages": [msg] }))
            .send()
            .await
            .map_err(|e| AdapterError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(PlatformMessageId(uuid::Uuid::new_v4().to_string()))
        } else {
            Err(AdapterError::Platform(response.text().await.unwrap_or_default()))
        }
    }

    async fn get_message_status(&self, _: &str) -> AdapterResult<MessageStatus> { Ok(MessageStatus::Sent) }
}
