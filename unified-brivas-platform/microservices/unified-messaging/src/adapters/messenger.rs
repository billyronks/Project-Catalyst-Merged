//! Facebook Messenger adapter

use async_trait::async_trait;
use serde_json::json;

use super::{
    AdapterError, AdapterResult, ContentSupport, MediaType, MessageStatus, PlatformAdapter,
    PlatformCapabilities, PlatformMessageId,
};
use crate::model::{MessageContent, Platform, UnifiedMessage};

pub struct MessengerAdapter {
    page_access_token: String,
    http_client: reqwest::Client,
}

impl MessengerAdapter {
    pub fn new(page_access_token: String) -> Self {
        Self {
            page_access_token,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PlatformAdapter for MessengerAdapter {
    fn platform(&self) -> Platform {
        Platform::FacebookMessenger
    }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            max_text_length: 2000,
            supports_buttons: true,
            max_buttons: 3,
            supports_carousel: true,
            supports_quick_replies: true,
            supports_templates: true,
            supports_reactions: true,
            supports_read_receipts: true,
            supports_typing_indicator: true,
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Video,
                MediaType::Audio,
                MediaType::Document,
            ],
            max_media_size_bytes: 25 * 1024 * 1024,
        }
    }

    fn supports_content(&self, content: &MessageContent) -> ContentSupport {
        match content {
            MessageContent::Text { .. }
            | MessageContent::Image { .. }
            | MessageContent::Video { .. }
            | MessageContent::Interactive { .. } => ContentSupport::Full,
            _ => ContentSupport::Partial { unsupported: vec![] },
        }
    }

    async fn send_message(&self, message: &UnifiedMessage) -> AdapterResult<PlatformMessageId> {
        let payload = match &message.content {
            MessageContent::Text { body } => json!({
                "recipient": { "id": message.recipient_id },
                "message": { "text": body }
            }),
            MessageContent::Image { url, .. } => json!({
                "recipient": { "id": message.recipient_id },
                "message": {
                    "attachment": {
                        "type": "image",
                        "payload": { "url": url, "is_reusable": true }
                    }
                }
            }),
            MessageContent::Interactive { body, action, .. } => {
                let buttons: Vec<_> = action.buttons.as_ref()
                    .map(|btns| btns.iter().map(|b| json!({
                        "type": "postback",
                        "title": b.title,
                        "payload": b.id
                    })).collect())
                    .unwrap_or_default();
                json!({
                    "recipient": { "id": message.recipient_id },
                    "message": {
                        "attachment": {
                            "type": "template",
                            "payload": {
                                "template_type": "button",
                                "text": body,
                                "buttons": buttons
                            }
                        }
                    }
                })
            }
            _ => json!({
                "recipient": { "id": message.recipient_id },
                "message": { "text": "Unsupported" }
            }),
        };

        let response = self.http_client
            .post("https://graph.facebook.com/v18.0/me/messages")
            .query(&[("access_token", &self.page_access_token)])
            .json(&payload)
            .send()
            .await
            .map_err(|e| AdapterError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| AdapterError::Parse(e.to_string()))?;
        
        Ok(PlatformMessageId(result["message_id"].as_str().unwrap_or("unknown").to_string()))
    }

    async fn get_message_status(&self, _id: &str) -> AdapterResult<MessageStatus> {
        Ok(MessageStatus::Sent)
    }
}
