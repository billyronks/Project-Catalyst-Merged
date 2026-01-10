//! HTTP handlers for Voice Switch API

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::carrier::{
    Carrier, CarrierRepository, CarrierStats, CarrierSummary, CreateCarrierRequest,
    UpdateCarrierRequest,
};
use crate::kdb::{
    ActiveCall, CarrierKdbStats, DestinationStats, FraudAlert, QosMetrics, TrafficStats,
};
use crate::lcr::{LcrEngine, RoutingDecision, RoutingMode};
use crate::webrtc::{CreateSessionRequest, Session, SdpPayload, IceCandidate};
use crate::{AppState, Error, Result};

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub kdb_connected: bool,
}

/// Ready check response
#[derive(Serialize)]
pub struct ReadyResponse {
    pub ready: bool,
    pub database: bool,
    pub kdb: bool,
}

/// Stats response
#[derive(Serialize)]
pub struct StatsResponse {
    pub uptime_secs: u64,
    pub requests_total: u64,
    pub active_connections: u64,
}

// ============================================
// Health & Metrics Handlers
// ============================================

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let kdb_connected = state.kdb_client.health_check().await;

    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "voice-switch".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        kdb_connected,
    })
}

pub async fn ready(State(state): State<AppState>) -> Json<ReadyResponse> {
    let db_ok = state.db.get().await.is_ok();
    let kdb_ok = state.kdb_client.health_check().await;

    Json(ReadyResponse {
        ready: db_ok,
        database: db_ok,
        kdb: kdb_ok,
    })
}

pub async fn stats() -> Json<StatsResponse> {
    Json(StatsResponse {
        uptime_secs: 0, // TODO: track uptime
        requests_total: 0,
        active_connections: 0,
    })
}

// ============================================
// Carrier Handlers
// ============================================

pub async fn list_carriers(State(state): State<AppState>) -> Result<Json<Vec<Carrier>>> {
    let repo = CarrierRepository::new(&state.db);
    let carriers = repo.find_all().await?;
    Ok(Json(carriers))
}

pub async fn create_carrier(
    State(state): State<AppState>,
    Json(req): Json<CreateCarrierRequest>,
) -> Result<(StatusCode, Json<Carrier>)> {
    let repo = CarrierRepository::new(&state.db);
    let carrier = repo.create(req).await?;
    
    // Update cache
    state.carrier_cache.insert(carrier.clone());
    
    Ok((StatusCode::CREATED, Json(carrier)))
}

pub async fn get_carrier(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Carrier>> {
    // Try cache first
    if let Some(carrier) = state.carrier_cache.get(&id) {
        return Ok(Json(carrier));
    }

    let repo = CarrierRepository::new(&state.db);
    let carrier = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| Error::CarrierNotFound(id.to_string()))?;

    // Update cache
    state.carrier_cache.insert(carrier.clone());

    Ok(Json(carrier))
}

pub async fn update_carrier(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCarrierRequest>,
) -> Result<Json<Carrier>> {
    let repo = CarrierRepository::new(&state.db);
    let carrier = repo.update(id, req).await?;

    // Update cache
    state.carrier_cache.insert(carrier.clone());

    Ok(Json(carrier))
}

pub async fn delete_carrier(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let repo = CarrierRepository::new(&state.db);
    repo.delete(id).await?;

    // Remove from cache
    state.carrier_cache.remove(&id);

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_carrier_stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CarrierKdbStats>>> {
    let stats = state.kdb_client.get_carrier_stats(Some(id)).await?;
    Ok(Json(stats))
}

pub async fn get_carriers_summary(
    State(state): State<AppState>,
) -> Result<Json<CarrierSummary>> {
    let repo = CarrierRepository::new(&state.db);
    let carriers = repo.find_all().await?;

    let total = carriers.len() as i64;
    let active = carriers
        .iter()
        .filter(|c| c.status == crate::carrier::CarrierStatus::Active)
        .count() as i64;

    let traffic = state.kdb_client.get_traffic_stats().await?;

    Ok(Json(CarrierSummary {
        total_carriers: total,
        active_carriers: active,
        inactive_carriers: total - active,
        total_active_calls: traffic.active_calls,
        total_capacity: carriers.iter().map(|c| c.max_channels as i64).sum(),
        overall_asr: state.kdb_client.get_asr().await.unwrap_or(0.0),
        overall_acd: traffic.avg_call_duration,
    }))
}

// ============================================
// Route Handlers
// ============================================

#[derive(Deserialize)]
pub struct CreateRouteRequest {
    pub carrier_id: Uuid,
    pub prefix: String,
    pub rate: f64,
    pub priority: Option<i32>,
}

#[derive(Serialize)]
pub struct Route {
    pub id: Uuid,
    pub carrier_id: Uuid,
    pub prefix: String,
    pub rate: f64,
    pub priority: i32,
    pub enabled: bool,
}

pub async fn list_routes(State(state): State<AppState>) -> Result<Json<Vec<Route>>> {
    let client = state.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    let rows = client
        .query(
            "SELECT id, carrier_id, prefix, rate, priority, enabled FROM routes ORDER BY prefix, priority",
            &[],
        )
        .await?;

    let routes: Vec<Route> = rows
        .iter()
        .map(|row| Route {
            id: row.get("id"),
            carrier_id: row.get("carrier_id"),
            prefix: row.get("prefix"),
            rate: row.get("rate"),
            priority: row.get("priority"),
            enabled: row.get("enabled"),
        })
        .collect();

    Ok(Json(routes))
}

pub async fn create_route(
    State(state): State<AppState>,
    Json(req): Json<CreateRouteRequest>,
) -> Result<(StatusCode, Json<Route>)> {
    let id = Uuid::new_v4();
    let client = state.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    client
        .execute(
            "INSERT INTO routes (id, carrier_id, prefix, rate, priority, enabled) VALUES ($1, $2, $3, $4, $5, true)",
            &[&id, &req.carrier_id, &req.prefix, &req.rate, &req.priority.unwrap_or(1)],
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(Route {
            id,
            carrier_id: req.carrier_id,
            prefix: req.prefix,
            rate: req.rate,
            priority: req.priority.unwrap_or(1),
            enabled: true,
        }),
    ))
}

pub async fn delete_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let client = state.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    let result = client
        .execute("DELETE FROM routes WHERE id = $1", &[&id])
        .await?;

    if result == 0 {
        return Err(Error::RouteNotFound(id.to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// LCR Handlers
// ============================================

#[derive(Deserialize)]
pub struct RouteQuery {
    pub destination: String,
    pub mode: Option<String>,
}

pub async fn route_call(
    State(state): State<AppState>,
    Query(query): Query<RouteQuery>,
) -> Result<Json<RoutingDecision>> {
    let mode = match query.mode.as_deref() {
        Some("cost") => RoutingMode::LeastCost,
        Some("quality") => RoutingMode::Quality,
        Some("balanced") => RoutingMode::Balanced,
        Some("priority") => RoutingMode::Priority,
        Some("round_robin") => RoutingMode::RoundRobin,
        _ => RoutingMode::LeastCost,
    };

    let lcr = LcrEngine::new(
        state.carrier_cache.clone(),
        state.kdb_client.clone(),
        state.db.clone(),
    );

    let decision = lcr.route(&query.destination, mode).await?;
    Ok(Json(decision))
}

// ============================================
// Analytics Handlers (kdb+)
// ============================================

pub async fn kdb_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let connected = state.kdb_client.health_check().await;
    Json(serde_json::json!({
        "connected": connected
    }))
}

pub async fn get_traffic(State(state): State<AppState>) -> Result<Json<TrafficStats>> {
    let stats = state.kdb_client.get_traffic_stats().await?;
    Ok(Json(stats))
}

pub async fn get_all_carrier_stats(
    State(state): State<AppState>,
) -> Result<Json<Vec<CarrierKdbStats>>> {
    let stats = state.kdb_client.get_carrier_stats(None).await?;
    Ok(Json(stats))
}

#[derive(Deserialize)]
pub struct DestinationQuery {
    pub prefix: Option<String>,
}

pub async fn get_destinations(
    State(state): State<AppState>,
    Query(query): Query<DestinationQuery>,
) -> Result<Json<Vec<DestinationStats>>> {
    let stats = state
        .kdb_client
        .get_destination_stats(query.prefix.as_deref())
        .await?;
    Ok(Json(stats))
}

pub async fn get_qos(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<QosMetrics>> {
    let metrics = state.kdb_client.get_qos_metrics(id).await?;
    Ok(Json(metrics))
}

#[derive(Deserialize)]
pub struct AlertsQuery {
    pub limit: Option<i32>,
}

pub async fn get_fraud_alerts(
    State(state): State<AppState>,
    Query(query): Query<AlertsQuery>,
) -> Result<Json<Vec<FraudAlert>>> {
    let alerts = state.kdb_client.get_fraud_alerts(query.limit).await?;
    Ok(Json(alerts))
}

pub async fn get_active_calls(State(state): State<AppState>) -> Result<Json<Vec<ActiveCall>>> {
    let calls = state.kdb_client.get_active_calls().await?;
    Ok(Json(calls))
}

#[derive(Serialize)]
pub struct MetricValue {
    pub value: f64,
}

pub async fn get_cps(State(state): State<AppState>) -> Result<Json<MetricValue>> {
    let cps = state.kdb_client.get_cps().await?;
    Ok(Json(MetricValue { value: cps }))
}

pub async fn get_asr(State(state): State<AppState>) -> Result<Json<MetricValue>> {
    let asr = state.kdb_client.get_asr().await?;
    Ok(Json(MetricValue { value: asr }))
}

pub async fn get_acd(State(state): State<AppState>) -> Result<Json<MetricValue>> {
    let acd = state.kdb_client.get_acd().await?;
    Ok(Json(MetricValue { value: acd }))
}

// ============================================
// WebRTC Session Handlers
// ============================================

pub async fn create_webrtc_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<Session>)> {
    let session = crate::webrtc::create_session(&state.db, req).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

pub async fn get_webrtc_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Session>> {
    let session = crate::webrtc::get_session(&state.db, session_id).await?;
    Ok(Json(session))
}

pub async fn delete_webrtc_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<StatusCode> {
    crate::webrtc::delete_session(&state.db, session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_local_sdp(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(sdp): Json<SdpPayload>,
) -> Result<StatusCode> {
    crate::webrtc::set_local_sdp(&state.db, session_id, &sdp.sdp).await?;
    Ok(StatusCode::OK)
}

pub async fn set_remote_sdp(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(sdp): Json<SdpPayload>,
) -> Result<StatusCode> {
    crate::webrtc::set_remote_sdp(&state.db, session_id, &sdp.sdp).await?;
    Ok(StatusCode::OK)
}

pub async fn add_local_ice(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(ice): Json<IceCandidate>,
) -> Result<StatusCode> {
    crate::webrtc::add_ice_candidate(&state.db, session_id, &ice, true).await?;
    Ok(StatusCode::OK)
}

pub async fn add_remote_ice(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(ice): Json<IceCandidate>,
) -> Result<StatusCode> {
    crate::webrtc::add_ice_candidate(&state.db, session_id, &ice, false).await?;
    Ok(StatusCode::OK)
}

pub async fn get_codecs() -> Json<Vec<String>> {
    Json(vec![
        "opus".to_string(),
        "g711u".to_string(),
        "g711a".to_string(),
        "g729".to_string(),
        "h264".to_string(),
        "vp8".to_string(),
        "vp9".to_string(),
    ])
}
