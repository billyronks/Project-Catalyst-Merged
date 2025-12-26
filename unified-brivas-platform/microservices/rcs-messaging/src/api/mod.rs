//! API module

pub mod rest;
pub mod webhook;

use axum::{Router, routing::{get, post, put, delete}};
use crate::RcsMessagingService;

pub fn create_router(_service: &RcsMessagingService) -> Router {
    Router::new()
        // Health endpoints
        .route("/health", get(rest::health_check))
        .route("/ready", get(rest::ready_check))
        
        // Agent management
        .route("/rcs/v1/agents", post(rest::create_agent))
        .route("/rcs/v1/agents", get(rest::list_agents))
        .route("/rcs/v1/agents/:id", get(rest::get_agent))
        .route("/rcs/v1/agents/:id", put(rest::update_agent))
        .route("/rcs/v1/agents/:id", delete(rest::delete_agent))
        
        // Messaging
        .route("/rcs/v1/agents/:agent_id/messages", post(rest::send_message))
        .route("/rcs/v1/agents/:agent_id/messages/rich-card", post(rest::send_rich_card))
        .route("/rcs/v1/agents/:agent_id/messages/carousel", post(rest::send_carousel))
        .route("/rcs/v1/messages/:id", get(rest::get_message))
        .route("/rcs/v1/messages/:id/revoke", post(rest::revoke_message))
        
        // Capability checking
        .route("/rcs/v1/capability", post(rest::check_capability))
        .route("/rcs/v1/capability/batch", post(rest::batch_check_capability))
        
        // Templates
        .route("/rcs/v1/templates", post(rest::create_template))
        .route("/rcs/v1/templates", get(rest::list_templates))
        .route("/rcs/v1/templates/:id", get(rest::get_template))
        
        // Webhooks
        .route("/rcs/v1/webhook", post(webhook::handle_webhook))
}
