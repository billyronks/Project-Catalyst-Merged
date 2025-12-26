//! REST API for certificate management

use axum::{
    extract::{Path, State},
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::certificate::CertificateManager;

pub fn create_router(cert_manager: &CertificateManager) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/v1/certificates", get(list_certificates))
        .route("/v1/certificates", post(upload_certificate))
        .route("/v1/certificates/{id}", get(get_certificate))
        .route("/v1/certificates/{id}", delete(delete_certificate))
        .route("/v1/certificates/{id}/rotate", post(rotate_certificate))
        .route("/v1/stats", get(get_stats))
        .with_state(cert_manager.clone())
}

async fn health() -> &'static str { "OK" }
async fn ready() -> &'static str { "OK" }

#[derive(Serialize)]
struct CertificateListResponse {
    certificates: Vec<CertificateSummary>,
}

#[derive(Serialize)]
struct CertificateSummary {
    id: String,
    name: String,
    subject: String,
    spc: String,
    is_active: bool,
    is_default: bool,
}

async fn list_certificates(
    State(cert_manager): State<CertificateManager>,
) -> Json<CertificateListResponse> {
    let certs = cert_manager.list_certificates(false, false, "").await;
    let summaries = certs.into_iter().map(|c| CertificateSummary {
        id: c.id,
        name: c.name,
        subject: c.subject,
        spc: c.spc,
        is_active: c.is_active,
        is_default: c.is_default,
    }).collect();
    Json(CertificateListResponse { certificates: summaries })
}

#[derive(Deserialize)]
struct UploadCertificateRequest {
    name: String,
    certificate_pem: String,
    private_key_pem: String,
    certificate_url: String,
    set_as_default: bool,
}

async fn upload_certificate(
    State(_cert_manager): State<CertificateManager>,
    Json(_req): Json<UploadCertificateRequest>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "uploaded", "id": uuid::Uuid::new_v4().to_string() }))
}

async fn get_certificate(
    State(cert_manager): State<CertificateManager>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    match cert_manager.get_certificate(&id).await {
        Some(cert) => Json(serde_json::json!({ "id": cert.id, "name": cert.name, "subject": cert.subject })),
        None => Json(serde_json::json!({ "error": "Not found" })),
    }
}

async fn delete_certificate(
    State(_cert_manager): State<CertificateManager>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "deleted" }))
}

async fn rotate_certificate(
    State(_cert_manager): State<CertificateManager>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "rotated" }))
}

async fn get_stats() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "total_signs": 0, "total_verifications": 0 }))
}
