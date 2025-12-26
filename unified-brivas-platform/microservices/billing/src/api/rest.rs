//! Billing REST API

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use axum::http::StatusCode;

use crate::{CdrCollector, InvoiceService, RatingEngine, WalletService};

#[derive(Clone)]
pub struct AppState {
    pub cdr_collector: CdrCollector,
    pub rating_engine: RatingEngine,
    pub invoice_service: InvoiceService,
    pub wallet_service: WalletService,
}

pub fn create_router(
    cdr_collector: CdrCollector,
    rating_engine: RatingEngine,
    invoice_service: InvoiceService,
    wallet_service: WalletService,
) -> Router {
    let state = AppState {
        cdr_collector,
        rating_engine,
        invoice_service,
        wallet_service,
    };

    Router::new()
        // Health
        .route("/health", get(health))
        .route("/ready", get(ready))
        // Wallets
        .route("/v1/wallets", post(create_wallet))
        .route("/v1/wallets/{id}", get(get_wallet))
        .route("/v1/wallets/{id}/balance", get(get_balance))
        .route("/v1/wallets/{id}/credit", post(credit_wallet))
        .route("/v1/wallets/{id}/debit", post(debit_wallet))
        .route("/v1/wallets/{id}/transactions", get(get_transactions))
        // Invoices
        .route("/v1/invoices", get(list_invoices))
        .route("/v1/invoices/{id}", get(get_invoice))
        .route("/v1/invoices/{id}/pay", post(pay_invoice))
        // Stats
        .route("/v1/stats", get(get_stats))
        .with_state(state)
}

async fn health() -> &'static str { "OK" }
async fn ready() -> &'static str { "OK" }

// Wallet endpoints

#[derive(Deserialize)]
struct CreateWalletRequest {
    customer_id: Uuid,
    currency: String,
    initial_balance: Option<Decimal>,
}

#[derive(Serialize)]
struct WalletResponse {
    id: Uuid,
    customer_id: Uuid,
    balance: Decimal,
    currency: String,
}

async fn create_wallet(
    State(state): State<AppState>,
    Json(req): Json<CreateWalletRequest>,
) -> Result<Json<WalletResponse>, (StatusCode, String)> {
    let wallet = state.wallet_service
        .create_wallet(req.customer_id, &req.currency, req.initial_balance.unwrap_or(Decimal::ZERO))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(WalletResponse {
        id: wallet.id,
        customer_id: wallet.customer_id,
        balance: wallet.balance,
        currency: wallet.currency,
    }))
}

async fn get_wallet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.wallet_service.get_wallet(id).await {
        Some(w) => Json(serde_json::json!({
            "id": w.id,
            "customer_id": w.customer_id,
            "balance": w.balance,
            "currency": w.currency
        })),
        None => Json(serde_json::json!({ "error": "Not found" })),
    }
}

async fn get_balance(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.wallet_service.check_balance(id).await {
        Ok(balance) => Json(serde_json::json!({ "balance": balance })),
        Err(_) => Json(serde_json::json!({ "error": "Wallet not found" })),
    }
}

#[derive(Deserialize)]
struct TransactionRequest {
    amount: Decimal,
    description: String,
}

async fn credit_wallet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransactionRequest>,
) -> Json<serde_json::Value> {
    match state.wallet_service.credit(id, req.amount, &req.description).await {
        Ok(balance) => Json(serde_json::json!({ "balance": balance })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn debit_wallet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransactionRequest>,
) -> Json<serde_json::Value> {
    match state.wallet_service.debit(id, req.amount, &req.description).await {
        Ok(balance) => Json(serde_json::json!({ "balance": balance })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn get_transactions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let txs = state.wallet_service.get_transactions(id, 50).await;
    Json(serde_json::json!({ "transactions": txs }))
}

// Invoice endpoints

#[derive(Deserialize)]
struct ListInvoicesQuery {
    customer_id: Uuid,
}

async fn list_invoices(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListInvoicesQuery>,
) -> Json<serde_json::Value> {
    let invoices = state.invoice_service.list_invoices(query.customer_id).await;
    Json(serde_json::json!({ "invoices": invoices }))
}

async fn get_invoice(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.invoice_service.get_invoice(id).await {
        Some(inv) => Json(serde_json::to_value(inv).unwrap_or_else(|_| serde_json::json!({"error": "Serialization failed"}))),
        None => Json(serde_json::json!({ "error": "Not found" })),
    }
}

async fn pay_invoice(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.invoice_service.mark_paid(id).await {
        Ok(()) => Json(serde_json::json!({ "status": "paid" })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn get_stats() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "total_cdrs": 0,
        "pending_cdrs": 0,
        "total_invoices": 0,
        "revenue_mtd": "0.00"
    }))
}
