//! Discord adapter

use async_trait::async_trait;
use serde_json::json;

use super::{
    AdapterError, AdapterResult, ContentSupport, MediaType, MessageStatus, PlatformAdapter,
    PlatformCapabilities, PlatformMessageId,
};
use crate::model::{MessageContent, Platform, UnifiedMessage};

pub struct DiscordAdapter {
    bot_token: String,
    http_client: reqwest::Client,
}

impl DiscordAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PlatformAdapter for DiscordAdapter {
    fn platform(&self) -> Platform {
        Platform::Discord
    }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            max_text_length: 2000,
            supports_buttons: true,
            max_buttons: 25,
            supports_carousel: false,
            supports_quick_replies: false,
            supports_templates: false,
            supports_reactions: true,
            supports_read_receipts: false,
            supports_typing_indicator: true,
            supported_media_types: vec![MediaType::Image, MediaType::Video, MediaType::Document],
            max_media_size_bytes: 8 * 1024 * 1024,
        }
    }

    fn supports_content(&self, content: &MessageContent) -> ContentSupport {
        match content {
            MessageContent::Text { .. } | MessageContent::Interactive { .. } => ContentSupport::Full,
            _ => ContentSupport::Partial { unsupported: vec![] },
        }
    }

    async fn send_message(&self, message: &UnifiedMessage) -> AdapterResult<PlatformMessageId> {
        let payload = match &message.content {
            MessageContent::Text { body } => json!({ "content": body }),
            MessageContent::Interactive { body, action, .. } => {
                let components: Vec<_> = action.buttons.as_ref()
                    .map(|btns| vec![json!({
                        "type": 1,
                        "components": btns.iter().map(|b| json!({
                            "type": 2,
                            "style": 1,
                            "label": b.title,
                            "custom_id": b.id
                        })).collect::<Vec<_>>()
                    })])
                    .unwrap_or_default();
                json!({ "content": body, "components": components })
            }
            _ => json!({ "content": "Unsupported message type" }),
        };

        let response = self.http_client
            .post(format!("https://discord.com/api/v10/channels/{}/messages", message.recipient_id))
            .header("Authorization", format!("Bot {}", self.bot_token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| AdapterError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| AdapterError::Parse(e.to_string()))?;
        
        Ok(PlatformMessageId(result["id"].as_str().unwrap_or("unknown").to_string()))
    }

    async fn get_message_status(&self, _id: &str) -> AdapterResult<MessageStatus> {
        Ok(MessageStatus::Sent)
    }
}
