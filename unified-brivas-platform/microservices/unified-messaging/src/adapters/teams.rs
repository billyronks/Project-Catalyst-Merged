//! Microsoft Teams adapter

use async_trait::async_trait;
use serde_json::json;

use super::{
    AdapterError, AdapterResult, ContentSupport, MediaType, MessageStatus, PlatformAdapter,
    PlatformCapabilities, PlatformMessageId,
};
use crate::model::{MessageContent, Platform, UnifiedMessage};

pub struct TeamsAdapter {
    webhook_url: String,
    http_client: reqwest::Client,
}

impl TeamsAdapter {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url, http_client: reqwest::Client::new() }
    }
}

#[async_trait]
impl PlatformAdapter for TeamsAdapter {
    fn platform(&self) -> Platform { Platform::MicrosoftTeams }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            max_text_length: 28000,
            supports_buttons: true,
            max_buttons: 6,
            supports_carousel: true,
            supports_quick_replies: false,
            supports_templates: true,
            supports_reactions: true,
            supports_read_receipts: true,
            supports_typing_indicator: false,
            supported_media_types: vec![MediaType::Image, MediaType::Document],
            max_media_size_bytes: 25 * 1024 * 1024,
        }
    }

    fn supports_content(&self, _: &MessageContent) -> ContentSupport { ContentSupport::Full }

    async fn send_message(&self, message: &UnifiedMessage) -> AdapterResult<PlatformMessageId> {
        let card = match &message.content {
            MessageContent::Text { body } => json!({
                "type": "message",
                "attachments": [{
                    "contentType": "application/vnd.microsoft.card.adaptive",
                    "content": {
                        "type": "AdaptiveCard",
                        "version": "1.4",
                        "body": [{ "type": "TextBlock", "text": body, "wrap": true }]
                    }
                }]
            }),
            MessageContent::Interactive { body, action, .. } => {
                let actions: Vec<_> = action.buttons.as_ref().map(|b| b.iter().map(|btn| json!({
                    "type": "Action.Submit", "title": btn.title, "data": { "id": btn.id }
                })).collect()).unwrap_or_default();
                json!({
                    "type": "message",
                    "attachments": [{
                        "contentType": "application/vnd.microsoft.card.adaptive",
                        "content": {
                            "type": "AdaptiveCard",
                            "version": "1.4",
                            "body": [{ "type": "TextBlock", "text": body, "wrap": true }],
                            "actions": actions
                        }
                    }]
                })
            }
            _ => json!({ "type": "message", "text": "Unsupported" }),
        };

        let response = self.http_client.post(&self.webhook_url).json(&card).send().await
            .map_err(|e| AdapterError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(PlatformMessageId(uuid::Uuid::new_v4().to_string()))
        } else {
            Err(AdapterError::Platform(response.text().await.unwrap_or_default()))
        }
    }

    async fn get_message_status(&self, _: &str) -> AdapterResult<MessageStatus> { Ok(MessageStatus::Sent) }
}
