//! User Service REST API

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AuthService, IdentityService, MfaService};

#[derive(Clone)]
pub struct AppState {
    pub identity_service: IdentityService,
    pub auth_service: AuthService,
    pub mfa_service: MfaService,
}

pub fn create_router(
    identity_service: IdentityService,
    auth_service: AuthService,
    mfa_service: MfaService,
) -> Router {
    let state = AppState {
        identity_service,
        auth_service,
        mfa_service,
    };

    Router::new()
        // Health
        .route("/health", get(health))
        .route("/ready", get(ready))
        // Auth
        .route("/v1/auth/register", post(register))
        .route("/v1/auth/login", post(login))
        .route("/v1/auth/refresh", post(refresh_token))
        .route("/v1/auth/logout", post(logout))
        // Users
        .route("/v1/users/{id}", get(get_user))
        .route("/v1/users/{id}", axum::routing::patch(update_user))
        // MFA
        .route("/v1/mfa/setup", post(setup_mfa))
        .route("/v1/mfa/verify", post(verify_mfa))
        .route("/v1/mfa/disable", post(disable_mfa))
        .with_state(state)
}

async fn health() -> &'static str { "OK" }
async fn ready() -> &'static str { "OK" }

// Auth endpoints

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Serialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    token_type: String,
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Json<serde_json::Value> {
    let tenant_id = Uuid::new_v4(); // Default tenant for self-registration
    
    match state.identity_service.create_user(
        &req.email,
        &req.password,
        req.first_name.as_deref(),
        req.last_name.as_deref(),
        tenant_id,
    ).await {
        Ok(user) => {
            let token = state.auth_service.generate_token(
                user.id,
                &user.email,
                user.roles.clone(),
                user.tenant_id,
            ).unwrap();
            
            Json(serde_json::json!({
                "access_token": token,
                "refresh_token": state.auth_service.generate_refresh_token(),
                "expires_in": 3600,
                "token_type": "Bearer",
                "user": {
                    "id": user.id,
                    "email": user.email
                }
            }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    mfa_code: Option<String>,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Json<serde_json::Value> {
    let user = match state.identity_service.get_user_by_email(&req.email).await {
        Some(u) => u,
        None => return Json(serde_json::json!({ "error": "Invalid credentials" })),
    };

    // Verify password
    let valid = state.identity_service
        .verify_password(&req.password, &user.password_hash)
        .unwrap_or(false);
    
    if !valid {
        return Json(serde_json::json!({ "error": "Invalid credentials" }));
    }

    // Check MFA
    if user.mfa_enabled {
        if let Some(code) = &req.mfa_code {
            if !state.mfa_service.verify_totp(user.id, code).await.unwrap_or(false) {
                return Json(serde_json::json!({ "error": "Invalid MFA code" }));
            }
        } else {
            return Json(serde_json::json!({ "mfa_required": true }));
        }
    }

    // Record login
    let _ = state.identity_service.record_login(user.id).await;

    // Generate tokens
    let token = state.auth_service.generate_token(
        user.id,
        &user.email,
        user.roles.clone(),
        user.tenant_id,
    ).unwrap();

    Json(serde_json::json!({
        "access_token": token,
        "refresh_token": state.auth_service.generate_refresh_token(),
        "expires_in": 3600,
        "token_type": "Bearer"
    }))
}

async fn refresh_token() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "error": "Not implemented" }))
}

async fn logout() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "logged out" }))
}

// User endpoints

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.identity_service.get_user(id).await {
        Some(user) => Json(serde_json::json!({
            "id": user.id,
            "email": user.email,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "status": user.status,
            "mfa_enabled": user.mfa_enabled
        })),
        None => Json(serde_json::json!({ "error": "User not found" })),
    }
}

async fn update_user(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "error": "Not implemented" }))
}

// MFA endpoints

async fn setup_mfa(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let user_id = Uuid::parse_str(req["user_id"].as_str().unwrap_or("")).unwrap_or(Uuid::nil());
    let email = req["email"].as_str().unwrap_or("user@example.com");
    
    match state.mfa_service.enable_totp(user_id, email).await {
        Ok(setup) => Json(serde_json::json!({
            "secret": setup.secret,
            "qr_url": setup.qr_url,
            "backup_codes": setup.backup_codes
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn verify_mfa(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let user_id = Uuid::parse_str(req["user_id"].as_str().unwrap_or("")).unwrap_or(Uuid::nil());
    let code = req["code"].as_str().unwrap_or("");
    
    match state.mfa_service.confirm_totp(user_id, code).await {
        Ok(true) => Json(serde_json::json!({ "verified": true })),
        Ok(false) => Json(serde_json::json!({ "error": "Invalid code" })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn disable_mfa(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let user_id = Uuid::parse_str(req["user_id"].as_str().unwrap_or("")).unwrap_or(Uuid::nil());
    
    match state.mfa_service.disable_mfa(user_id).await {
        Ok(()) => Json(serde_json::json!({ "disabled": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
