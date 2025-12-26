//! Telegram Bot API adapter

use async_trait::async_trait;
use serde_json::json;

use super::{
    AdapterError, AdapterResult, ContentSupport, MediaType, MessageStatus, PlatformAdapter,
    PlatformCapabilities, PlatformMessageId,
};
use crate::model::{MessageContent, Platform, UnifiedMessage};

pub struct TelegramAdapter {
    bot_token: String,
    http_client: reqwest::Client,
}

impl TelegramAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            http_client: reqwest::Client::new(),
        }
    }

    fn get_method_for_content(&self, content: &MessageContent) -> &'static str {
        match content {
            MessageContent::Text { .. } => "sendMessage",
            MessageContent::Image { .. } => "sendPhoto",
            MessageContent::Video { .. } => "sendVideo",
            MessageContent::Audio { .. } => "sendAudio",
            MessageContent::Document { .. } => "sendDocument",
            MessageContent::Location { .. } => "sendLocation",
            MessageContent::Sticker { .. } => "sendSticker",
            MessageContent::Interactive { .. } => "sendMessage",
            _ => "sendMessage",
        }
    }

    fn to_platform_format(&self, message: &UnifiedMessage) -> serde_json::Value {
        let mut payload = json!({
            "chat_id": message.recipient_id,
        });

        match &message.content {
            MessageContent::Text { body } => {
                payload["text"] = json!(body);
            }
            MessageContent::Image { url, caption } => {
                payload["photo"] = json!(url);
                if let Some(cap) = caption {
                    payload["caption"] = json!(cap);
                }
            }
            MessageContent::Video { url, caption } => {
                payload["video"] = json!(url);
                if let Some(cap) = caption {
                    payload["caption"] = json!(cap);
                }
            }
            MessageContent::Audio { url } => {
                payload["audio"] = json!(url);
            }
            MessageContent::Document { url, filename } => {
                payload["document"] = json!(url);
                payload["caption"] = json!(filename);
            }
            MessageContent::Location { latitude, longitude, .. } => {
                payload["latitude"] = json!(latitude);
                payload["longitude"] = json!(longitude);
            }
            MessageContent::Interactive { body, action, .. } => {
                payload["text"] = json!(body);
                
                // Build inline keyboard
                if let Some(buttons) = &action.buttons {
                    let keyboard: Vec<Vec<serde_json::Value>> = buttons
                        .iter()
                        .map(|b| vec![json!({"text": b.title, "callback_data": b.id})])
                        .collect();
                    payload["reply_markup"] = json!({
                        "inline_keyboard": keyboard
                    });
                }
            }
            _ => {
                payload["text"] = json!("Unsupported message type");
            }
        }

        payload
    }
}

#[async_trait]
impl PlatformAdapter for TelegramAdapter {
    fn platform(&self) -> Platform {
        Platform::Telegram
    }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            max_text_length: 4096,
            supports_buttons: true,
            max_buttons: 100, // Telegram supports large inline keyboards
            supports_carousel: false,
            supports_quick_replies: true,
            supports_templates: false,
            supports_reactions: true,
            supports_read_receipts: false,
            supports_typing_indicator: true,
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Video,
                MediaType::Audio,
                MediaType::Document,
                MediaType::Sticker,
            ],
            max_media_size_bytes: 50 * 1024 * 1024,
        }
    }

    fn supports_content(&self, content: &MessageContent) -> ContentSupport {
        match content {
            MessageContent::Text { .. }
            | MessageContent::Image { .. }
            | MessageContent::Video { .. }
            | MessageContent::Audio { .. }
            | MessageContent::Document { .. }
            | MessageContent::Location { .. }
            | MessageContent::Sticker { .. }
            | MessageContent::Interactive { .. } => ContentSupport::Full,
            MessageContent::Template { .. } => ContentSupport::None {
                reason: "Telegram doesn't support templates".to_string(),
            },
            _ => ContentSupport::Partial {
                unsupported: vec!["carousel".to_string()],
            },
        }
    }

    async fn send_message(&self, message: &UnifiedMessage) -> AdapterResult<PlatformMessageId> {
        let method = self.get_method_for_content(&message.content);
        let payload = self.to_platform_format(message);

        let response = self
            .http_client
            .post(format!(
                "https://api.telegram.org/bot{}/{}",
                self.bot_token, method
            ))
            .json(&payload)
            .send()
            .await
            .map_err(|e| AdapterError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| AdapterError::Parse(e.to_string()))?;

            let message_id = result["result"]["message_id"]
                .as_i64()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            Ok(PlatformMessageId(message_id))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(AdapterError::Platform(error_text))
        }
    }

    async fn get_message_status(&self, _platform_message_id: &str) -> AdapterResult<MessageStatus> {
        // Telegram doesn't provide read receipts to bots
        Ok(MessageStatus::Sent)
    }
}
