//! Slack adapter

use async_trait::async_trait;
use serde_json::json;

use super::{
    AdapterError, AdapterResult, ContentSupport, MediaType, MessageStatus, PlatformAdapter,
    PlatformCapabilities, PlatformMessageId,
};
use crate::model::{MessageContent, Platform, UnifiedMessage};

pub struct SlackAdapter {
    bot_token: String,
    http_client: reqwest::Client,
}

impl SlackAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PlatformAdapter for SlackAdapter {
    fn platform(&self) -> Platform {
        Platform::Slack
    }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            max_text_length: 40000,
            supports_buttons: true,
            max_buttons: 25,
            supports_carousel: false,
            supports_quick_replies: true,
            supports_templates: true,
            supports_reactions: true,
            supports_read_receipts: false,
            supports_typing_indicator: false,
            supported_media_types: vec![MediaType::Image, MediaType::Document],
            max_media_size_bytes: 1024 * 1024 * 1024, // 1GB
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
            MessageContent::Text { body } => json!({
                "channel": message.recipient_id,
                "text": body
            }),
            MessageContent::Interactive { body, action, .. } => {
                let blocks: Vec<_> = vec![
                    json!({"type": "section", "text": {"type": "mrkdwn", "text": body}}),
                    json!({
                        "type": "actions",
                        "elements": action.buttons.as_ref().map(|btns| 
                            btns.iter().map(|b| json!({
                                "type": "button",
                                "text": {"type": "plain_text", "text": b.title},
                                "action_id": b.id
                            })).collect::<Vec<_>>()
                        ).unwrap_or_default()
                    })
                ];
                json!({ "channel": message.recipient_id, "blocks": blocks })
            }
            _ => json!({ "channel": message.recipient_id, "text": "Unsupported" }),
        };

        let response = self.http_client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&self.bot_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AdapterError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| AdapterError::Parse(e.to_string()))?;
        
        Ok(PlatformMessageId(result["ts"].as_str().unwrap_or("unknown").to_string()))
    }

    async fn get_message_status(&self, _id: &str) -> AdapterResult<MessageStatus> {
        Ok(MessageStatus::Sent)
    }
}
