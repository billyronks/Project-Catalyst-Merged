//! WebSocket handler for real-time messaging

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WebSocket upgrade handler
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // Send welcome message
    let welcome = WsMessage::Connected {
        connection_id: Uuid::new_v4().to_string(),
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    let response = handle_client_message(client_msg).await;
                    if let Ok(json) = serde_json::to_string(&response) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }
}

/// Server-to-client WebSocket messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Connected { connection_id: String },
    MessageReceived { message: serde_json::Value },
    TypingIndicator { conversation_id: String, user_id: String, is_typing: bool },
    PresenceUpdate { user_id: String, status: String },
    ReadReceipt { conversation_id: String, message_id: String, user_id: String },
    Ack { request_id: String },
    Error { code: String, message: String },
}

/// Client-to-server WebSocket messages
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Subscribe { conversation_ids: Vec<String> },
    Unsubscribe { conversation_ids: Vec<String> },
    SendMessage { request_id: String, conversation_id: String, content: serde_json::Value },
    Typing { conversation_id: String, is_typing: bool },
    MarkRead { conversation_id: String, message_id: String },
    Ping,
}

async fn handle_client_message(msg: ClientMessage) -> WsMessage {
    match msg {
        ClientMessage::Subscribe { .. } => WsMessage::Ack {
            request_id: "sub".to_string(),
        },
        ClientMessage::Unsubscribe { .. } => WsMessage::Ack {
            request_id: "unsub".to_string(),
        },
        ClientMessage::SendMessage { request_id, .. } => WsMessage::Ack { request_id },
        ClientMessage::Typing { conversation_id, is_typing } => WsMessage::TypingIndicator {
            conversation_id,
            user_id: "self".to_string(),
            is_typing,
        },
        ClientMessage::MarkRead { conversation_id, message_id } => WsMessage::ReadReceipt {
            conversation_id,
            message_id,
            user_id: "self".to_string(),
        },
        ClientMessage::Ping => WsMessage::Ack {
            request_id: "pong".to_string(),
        },
    }
}
