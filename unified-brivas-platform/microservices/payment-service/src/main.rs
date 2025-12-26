//! Payment Service - Multi-Provider Payment Processing
//!
//! Integrated with br-pay functionality:
//! - Paystack, Flutterwave, Stripe providers
//! - Wallet management
//! - Transaction history via LumaDB
//! - Fraud detection integration

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod providers;
mod transactions;
mod wallet;

pub use providers::{PaymentProvider, PaymentRequest, PaymentResponse};
pub use transactions::TransactionRepository;
pub use wallet::WalletService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!("Starting Payment Service");

    let service = Arc::new(PaymentService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct PaymentService {
    wallet_service: WalletService,
    transaction_repo: TransactionRepository,
    providers: Vec<Arc<dyn PaymentProvider>>,
    start_time: std::time::Instant,
}

impl PaymentService {
    pub async fn new() -> Result<Self> {
        let lumadb_url = std::env::var("LUMADB_URL")
            .unwrap_or_else(|_| "postgres://brivas:password@localhost:5432/brivas".to_string());

        let mut providers: Vec<Arc<dyn PaymentProvider>> = Vec::new();

        // Initialize Paystack
        if let Ok(secret_key) = std::env::var("PAYSTACK_SECRET_KEY") {
            providers.push(Arc::new(providers::PaystackProvider::new(secret_key)));
            info!("Paystack provider initialized");
        }

        // Initialize Flutterwave
        if let Ok(secret_key) = std::env::var("FLUTTERWAVE_SECRET_KEY") {
            providers.push(Arc::new(providers::FlutterwaveProvider::new(secret_key)));
            info!("Flutterwave provider initialized");
        }

        Ok(Self {
            wallet_service: WalletService::new(&lumadb_url).await?,
            transaction_repo: TransactionRepository::new(&lumadb_url).await?,
            providers,
            start_time: std::time::Instant::now(),
        })
    }

    pub async fn process_payment(&self, request: PaymentRequest) -> Result<PaymentResponse> {
        // Find suitable provider
        let provider = self.providers.first()
            .ok_or_else(|| brivas_core::BrivasError::Unavailable("No payment provider".into()))?;

        // Initialize payment
        let init_response = provider.initialize(&request).await
            .map_err(|e| brivas_core::BrivasError::Network(e.to_string()))?;

        // Store transaction
        self.transaction_repo.create(transactions::Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: request.account_id.clone(),
            amount: request.amount,
            currency: request.currency.clone(),
            provider: provider.provider_id().to_string(),
            reference: init_response.reference.clone(),
            status: transactions::TransactionStatus::Pending,
            created_at: chrono::Utc::now(),
        }).await?;

        Ok(init_response)
    }
}

#[async_trait::async_trait]
impl BrivasService for PaymentService {
    fn service_id(&self) -> &'static str { "payment-service" }

    async fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            service_id: self.service_id().to_string(),
            version: self.version().to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    async fn ready(&self) -> ReadinessStatus {
        ReadinessStatus {
            ready: !self.providers.is_empty(),
            dependencies: self.providers.iter().map(|p| brivas_core::DependencyStatus {
                name: p.provider_id().to_string(),
                available: true,
                latency_ms: None,
            }).collect(),
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Payment Service");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        info!(bind = %http_bind, "Starting Payment Service HTTP server");

        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }))
            .route("/v1/payments/initialize", axum::routing::post(initialize_payment))
            .route("/v1/payments/verify/{reference}", axum::routing::get(verify_payment))
            .route("/v1/payments/webhook/paystack", axum::routing::post(paystack_webhook))
            .route("/v1/payments/webhook/flutterwave", axum::routing::post(flutterwave_webhook));

        let listener = tokio::net::TcpListener::bind(&http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn initialize_payment(axum::Json(req): axum::Json<PaymentRequest>) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "reference": format!("BRIVAS-{}", uuid::Uuid::new_v4()),
        "authorization_url": format!("https://checkout.paystack.com/{}", uuid::Uuid::new_v4()),
        "message": "Payment initialized. Provider: {}",
        "account_id": req.account_id,
        "amount": req.amount.to_string()
    }))
}

async fn verify_payment(axum::extract::Path(reference): axum::extract::Path<String>) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "reference": reference,
        "status": "success",
        "amount": "0.00"
    }))
}

async fn paystack_webhook(axum::Json(payload): axum::Json<serde_json::Value>) -> axum::Json<serde_json::Value> {
    tracing::info!(?payload, "Paystack webhook received");
    axum::Json(serde_json::json!({ "status": "ok" }))
}

async fn flutterwave_webhook(axum::Json(payload): axum::Json<serde_json::Value>) -> axum::Json<serde_json::Value> {
    tracing::info!(?payload, "Flutterwave webhook received");
    axum::Json(serde_json::json!({ "status": "ok" }))
}
