//! Webhook handler for RCS delivery/status callbacks

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tracing::info;

/// RCS Webhook payload
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    pub agent_id: String,
    pub event_type: String,
    pub message_id: Option<String>,
    pub conversation_id: Option<String>,
    pub sender_phone_number: Option<String>,
    pub event: WebhookEvent,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WebhookEvent {
    #[serde(rename = "MESSAGE")]
    Message { text: Option<String>, suggestion_response: Option<SuggestionResponse> },
    #[serde(rename = "DELIVERY")]
    Delivery { status: String },
    #[serde(rename = "READ")]
    Read {},
    #[serde(rename = "IS_TYPING")]
    IsTyping {},
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionResponse {
    pub text: String,
    pub postback_data: String,
}

/// Handle incoming webhook
pub async fn handle_webhook(Json(payload): Json<WebhookPayload>) -> StatusCode {
    info!(
        agent_id = %payload.agent_id,
        event_type = %payload.event_type,
        "Received RCS webhook"
    );

    match payload.event {
        WebhookEvent::Message { text, suggestion_response } => {
            if let Some(text) = text {
                info!(text = %text, "Received message from user");
                // TODO: Forward to conversation handler
            }
            if let Some(response) = suggestion_response {
                info!(
                    text = %response.text,
                    postback = %response.postback_data,
                    "Received suggestion response"
                );
                // TODO: Handle postback
            }
        }
        WebhookEvent::Delivery { status } => {
            info!(status = %status, "Message delivery status update");
            // TODO: Update message status in store
        }
        WebhookEvent::Read {} => {
            info!("Message read by user");
            // TODO: Update read status
        }
        WebhookEvent::IsTyping {} => {
            info!("User is typing");
            // TODO: Forward typing indicator
        }
        WebhookEvent::Unknown => {
            info!("Unknown webhook event type");
        }
    }

    StatusCode::OK
}
