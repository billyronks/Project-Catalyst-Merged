//! WhatsApp Business API adapter

use async_trait::async_trait;
use serde_json::json;

use super::{
    AdapterError, AdapterResult, ContentSupport, MediaType, MessageStatus, PlatformAdapter,
    PlatformCapabilities, PlatformMessageId,
};
use crate::model::{MessageContent, Platform, UnifiedMessage};

pub struct WhatsAppAdapter {
    phone_number_id: String,
    access_token: String,
    http_client: reqwest::Client,
}

impl WhatsAppAdapter {
    pub fn new(phone_number_id: String, access_token: String) -> Self {
        Self {
            phone_number_id,
            access_token,
            http_client: reqwest::Client::new(),
        }
    }

    fn to_platform_format(&self, message: &UnifiedMessage) -> serde_json::Value {
        let mut payload = json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": message.recipient_id,
        });

        match &message.content {
            MessageContent::Text { body } => {
                payload["type"] = json!("text");
                payload["text"] = json!({ "body": body });
            }
            MessageContent::Image { url, caption } => {
                payload["type"] = json!("image");
                payload["image"] = json!({
                    "link": url,
                    "caption": caption
                });
            }
            MessageContent::Video { url, caption } => {
                payload["type"] = json!("video");
                payload["video"] = json!({
                    "link": url,
                    "caption": caption
                });
            }
            MessageContent::Document { url, filename } => {
                payload["type"] = json!("document");
                payload["document"] = json!({
                    "link": url,
                    "filename": filename
                });
            }
            MessageContent::Location { latitude, longitude, name } => {
                payload["type"] = json!("location");
                payload["location"] = json!({
                    "latitude": latitude,
                    "longitude": longitude,
                    "name": name
                });
            }
            MessageContent::Template { template_name, language, components } => {
                payload["type"] = json!("template");
                payload["template"] = json!({
                    "name": template_name,
                    "language": { "code": language },
                    "components": components.iter().map(|c| json!({
                        "type": c.component_type,
                        "parameters": c.parameters.iter().map(|p| json!({
                            "type": p.param_type,
                            "text": p.text
                        })).collect::<Vec<_>>()
                    })).collect::<Vec<_>>()
                });
            }
            MessageContent::Interactive { interactive_type, body, action } => {
                payload["type"] = json!("interactive");
                let int_type = match interactive_type {
                    crate::model::InteractiveType::Button => "button",
                    crate::model::InteractiveType::List => "list",
                    _ => "button",
                };
                payload["interactive"] = json!({
                    "type": int_type,
                    "body": { "text": body },
                    "action": {
                        "buttons": action.buttons.as_ref().map(|btns| btns.iter().map(|b| json!({
                            "type": "reply",
                            "reply": { "id": b.id, "title": b.title }
                        })).collect::<Vec<_>>())
                    }
                });
            }
            _ => {
                payload["type"] = json!("text");
                payload["text"] = json!({ "body": "Unsupported message type" });
            }
        }

        payload
    }
}

#[async_trait]
impl PlatformAdapter for WhatsAppAdapter {
    fn platform(&self) -> Platform {
        Platform::WhatsApp
    }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            max_text_length: 4096,
            supports_buttons: true,
            max_buttons: 3,
            supports_carousel: false,
            supports_quick_replies: true,
            supports_templates: true,
            supports_reactions: true,
            supports_read_receipts: true,
            supports_typing_indicator: false,
            supported_media_types: vec![
                MediaType::Image,
                MediaType::Video,
                MediaType::Audio,
                MediaType::Document,
                MediaType::Sticker,
            ],
            max_media_size_bytes: 100 * 1024 * 1024,
        }
    }

    fn supports_content(&self, content: &MessageContent) -> ContentSupport {
        match content {
            MessageContent::Text { .. }
            | MessageContent::Image { .. }
            | MessageContent::Video { .. }
            | MessageContent::Document { .. }
            | MessageContent::Location { .. }
            | MessageContent::Template { .. }
            | MessageContent::Interactive { .. } => ContentSupport::Full,
            MessageContent::Sticker { .. } => ContentSupport::Full,
            MessageContent::Reaction { .. } => ContentSupport::Full,
            _ => ContentSupport::Partial {
                unsupported: vec!["contact".to_string()],
            },
        }
    }

    async fn send_message(&self, message: &UnifiedMessage) -> AdapterResult<PlatformMessageId> {
        let payload = self.to_platform_format(message);

        let response = self
            .http_client
            .post(format!(
                "https://graph.facebook.com/v18.0/{}/messages",
                self.phone_number_id
            ))
            .bearer_auth(&self.access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AdapterError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| AdapterError::Parse(e.to_string()))?;

            let message_id = result["messages"][0]["id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            Ok(PlatformMessageId(message_id))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(AdapterError::Platform(error_text))
        }
    }

    async fn get_message_status(&self, _platform_message_id: &str) -> AdapterResult<MessageStatus> {
        // WhatsApp sends status via webhooks, not polling
        Ok(MessageStatus::Sent)
    }
}
