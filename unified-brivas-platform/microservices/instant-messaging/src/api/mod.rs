//! API module - REST, WebSocket, GraphQL

pub mod rest;
pub mod websocket;
pub mod graphql;

use axum::{
    Router,
    routing::{get, post, put, delete},
};

use crate::InstantMessagingService;

pub fn create_router(_service: &InstantMessagingService) -> Router {
    Router::new()
        // Health endpoints
        .route("/health", get(rest::health_check))
        .route("/ready", get(rest::ready_check))
        
        // Conversation endpoints
        .route("/im/v1/conversations", post(rest::create_conversation))
        .route("/im/v1/conversations", get(rest::list_conversations))
        .route("/im/v1/conversations/:id", get(rest::get_conversation))
        .route("/im/v1/conversations/:id", delete(rest::delete_conversation))
        
        // Message endpoints
        .route("/im/v1/conversations/:id/messages", post(rest::send_message))
        .route("/im/v1/conversations/:id/messages", get(rest::get_messages))
        .route("/im/v1/messages/:id", put(rest::edit_message))
        .route("/im/v1/messages/:id", delete(rest::delete_message))
        .route("/im/v1/messages/:id/reactions", post(rest::add_reaction))
        
        // Presence endpoints
        .route("/im/v1/presence", get(rest::get_presence))
        .route("/im/v1/presence", put(rest::update_presence))
        .route("/im/v1/typing", post(rest::send_typing))
        .route("/im/v1/read-receipts", post(rest::send_read_receipts))
        
        // File endpoints
        .route("/im/v1/files/upload", post(rest::upload_file))
        .route("/im/v1/files/:id", get(rest::get_file))
        
        // Sync endpoint
        .route("/im/v1/sync", get(rest::sync_state))
        
        // WebSocket
        .route("/im/v1/ws", get(websocket::ws_handler))
}
