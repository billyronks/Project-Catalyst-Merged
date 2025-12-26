//! REST API handlers for RCS

use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generic API response
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(Self { success: true, data: Some(data), error: None })
    }

    pub fn error(message: impl Into<String>) -> Json<Self> {
        Json(Self { success: false, data: None, error: Some(message.into()) })
    }
}

// Health
pub async fn health_check() -> &'static str { "OK" }
pub async fn ready_check() -> &'static str { "OK" }

// Agent handlers
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub display_name: String,
    pub logo_url: String,
    pub primary_color: String,
    pub webhook_url: String,
}

#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub verification_status: String,
    pub created_at: String,
}

pub async fn create_agent(Json(req): Json<CreateAgentRequest>) -> Json<ApiResponse<AgentResponse>> {
    let response = AgentResponse {
        id: Uuid::new_v4(),
        name: req.name,
        display_name: req.display_name,
        verification_status: "pending".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    ApiResponse::success(response)
}

pub async fn list_agents() -> Json<ApiResponse<Vec<AgentResponse>>> {
    ApiResponse::success(vec![])
}

pub async fn get_agent(Path(id): Path<Uuid>) -> Json<ApiResponse<AgentResponse>> {
    let response = AgentResponse {
        id,
        name: "test-agent".to_string(),
        display_name: "Test Agent".to_string(),
        verification_status: "verified".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    ApiResponse::success(response)
}

pub async fn update_agent(
    Path(_id): Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> Json<ApiResponse<AgentResponse>> {
    let response = AgentResponse {
        id: Uuid::new_v4(),
        name: "updated-agent".to_string(),
        display_name: "Updated Agent".to_string(),
        verification_status: "verified".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    ApiResponse::success(response)
}

pub async fn delete_agent(Path(_id): Path<Uuid>) -> StatusCode {
    StatusCode::NO_CONTENT
}

// Message handlers
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub to: String,
    pub text: String,
    pub suggestions: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub recipient: String,
    pub status: String,
    pub rcs_enabled: bool,
    pub created_at: String,
}

pub async fn send_message(
    Path(agent_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Json<ApiResponse<MessageResponse>> {
    let response = MessageResponse {
        id: Uuid::new_v4(),
        agent_id,
        recipient: req.to,
        status: "sent".to_string(),
        rcs_enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    ApiResponse::success(response)
}

#[derive(Debug, Deserialize)]
pub struct SendRichCardRequest {
    pub to: String,
    pub card: serde_json::Value,
    pub fallback_text: Option<String>,
}

pub async fn send_rich_card(
    Path(agent_id): Path<Uuid>,
    Json(req): Json<SendRichCardRequest>,
) -> Json<ApiResponse<MessageResponse>> {
    let response = MessageResponse {
        id: Uuid::new_v4(),
        agent_id,
        recipient: req.to,
        status: "sent".to_string(),
        rcs_enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    ApiResponse::success(response)
}

#[derive(Debug, Deserialize)]
pub struct SendCarouselRequest {
    pub to: String,
    pub cards: Vec<serde_json::Value>,
    pub fallback_text: Option<String>,
}

pub async fn send_carousel(
    Path(agent_id): Path<Uuid>,
    Json(req): Json<SendCarouselRequest>,
) -> Json<ApiResponse<MessageResponse>> {
    let response = MessageResponse {
        id: Uuid::new_v4(),
        agent_id,
        recipient: req.to,
        status: "sent".to_string(),
        rcs_enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    ApiResponse::success(response)
}

pub async fn get_message(Path(id): Path<Uuid>) -> Json<ApiResponse<MessageResponse>> {
    let response = MessageResponse {
        id,
        agent_id: Uuid::new_v4(),
        recipient: "+1234567890".to_string(),
        status: "delivered".to_string(),
        rcs_enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    ApiResponse::success(response)
}

pub async fn revoke_message(Path(_id): Path<Uuid>) -> StatusCode {
    StatusCode::OK
}

// Capability handlers
#[derive(Debug, Deserialize)]
pub struct CapabilityRequest {
    pub phone_number: String,
}

#[derive(Debug, Serialize)]
pub struct CapabilityResponse {
    pub phone_number: String,
    pub rcs_enabled: bool,
    pub carrier: Option<String>,
    pub features: serde_json::Value,
}

pub async fn check_capability(Json(req): Json<CapabilityRequest>) -> Json<ApiResponse<CapabilityResponse>> {
    let response = CapabilityResponse {
        phone_number: req.phone_number,
        rcs_enabled: true,
        carrier: Some("MTN".to_string()),
        features: serde_json::json!({
            "richCard": true,
            "carousel": true,
            "fileTransfer": true
        }),
    };
    ApiResponse::success(response)
}

#[derive(Debug, Deserialize)]
pub struct BatchCapabilityRequest {
    pub phone_numbers: Vec<String>,
}

pub async fn batch_check_capability(
    Json(req): Json<BatchCapabilityRequest>,
) -> Json<ApiResponse<Vec<CapabilityResponse>>> {
    let responses: Vec<CapabilityResponse> = req.phone_numbers
        .into_iter()
        .map(|phone| CapabilityResponse {
            phone_number: phone,
            rcs_enabled: true,
            carrier: Some("MTN".to_string()),
            features: serde_json::json!({"richCard": true}),
        })
        .collect();
    ApiResponse::success(responses)
}

// Template handlers
pub async fn create_template(Json(_req): Json<serde_json::Value>) -> Json<ApiResponse<serde_json::Value>> {
    ApiResponse::success(serde_json::json!({
        "id": Uuid::new_v4(),
        "status": "created"
    }))
}

pub async fn list_templates() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    ApiResponse::success(vec![])
}

pub async fn get_template(Path(_id): Path<Uuid>) -> Json<ApiResponse<serde_json::Value>> {
    ApiResponse::success(serde_json::json!({
        "id": Uuid::new_v4(),
        "name": "welcome_card",
        "type": "rich_card"
    }))
}
