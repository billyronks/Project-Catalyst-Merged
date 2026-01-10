//! Router configuration for Voice Switch API

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers;
use crate::AppState;

/// Create the main router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health & Metrics
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
        .route("/stats", get(handlers::stats))
        // Carrier Management
        .route("/api/v1/carriers", get(handlers::list_carriers))
        .route("/api/v1/carriers", post(handlers::create_carrier))
        .route("/api/v1/carriers/:id", get(handlers::get_carrier))
        .route("/api/v1/carriers/:id", put(handlers::update_carrier))
        .route("/api/v1/carriers/:id", delete(handlers::delete_carrier))
        .route("/api/v1/carriers/:id/stats", get(handlers::get_carrier_stats))
        .route("/api/v1/carriers/summary", get(handlers::get_carriers_summary))
        // Route Management
        .route("/api/v1/routes", get(handlers::list_routes))
        .route("/api/v1/routes", post(handlers::create_route))
        .route("/api/v1/routes/:id", delete(handlers::delete_route))
        // LCR
        .route("/api/v1/lcr/route", get(handlers::route_call))
        // Analytics (kdb+)
        .route("/api/v1/kdb/health", get(handlers::kdb_health))
        .route("/api/v1/kdb/traffic", get(handlers::get_traffic))
        .route("/api/v1/kdb/carriers", get(handlers::get_all_carrier_stats))
        .route("/api/v1/kdb/carriers/:id", get(handlers::get_carrier_stats))
        .route("/api/v1/kdb/destinations", get(handlers::get_destinations))
        .route("/api/v1/kdb/qos/:id", get(handlers::get_qos))
        .route("/api/v1/kdb/fraud/alerts", get(handlers::get_fraud_alerts))
        .route("/api/v1/kdb/calls/active", get(handlers::get_active_calls))
        .route("/api/v1/kdb/metrics/cps", get(handlers::get_cps))
        .route("/api/v1/kdb/metrics/asr", get(handlers::get_asr))
        .route("/api/v1/kdb/metrics/acd", get(handlers::get_acd))
        // WebRTC Session Management
        .route("/api/v1/webrtc/session", post(handlers::create_webrtc_session))
        .route("/api/v1/webrtc/session/:sessionId", get(handlers::get_webrtc_session))
        .route("/api/v1/webrtc/session/:sessionId", delete(handlers::delete_webrtc_session))
        .route("/api/v1/webrtc/session/:sessionId/sdp/local", post(handlers::set_local_sdp))
        .route("/api/v1/webrtc/session/:sessionId/sdp/remote", post(handlers::set_remote_sdp))
        .route("/api/v1/webrtc/session/:sessionId/ice/local", post(handlers::add_local_ice))
        .route("/api/v1/webrtc/session/:sessionId/ice/remote", post(handlers::add_remote_ice))
        .route("/api/v1/webrtc/codecs", get(handlers::get_codecs))
        .with_state(state)
}
