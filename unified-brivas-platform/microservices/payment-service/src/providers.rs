//! Payment providers (Paystack, Flutterwave)

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub account_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub email: String,
    pub callback_url: Option<String>,
    pub metadata: serde_json::Value,
}

/// Payment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub reference: String,
    pub authorization_url: String,
    pub access_code: String,
}

/// Payment verification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub status: String,
    pub reference: String,
    pub amount: Decimal,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Payment provider trait
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn initialize(&self, request: &PaymentRequest) -> Result<PaymentResponse, ProviderError>;
    async fn verify(&self, reference: &str) -> Result<VerificationResponse, ProviderError>;
    async fn refund(&self, reference: &str, amount: Decimal) -> Result<String, ProviderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Verification failed: {0}")]
    Verification(String),
}

// ============== Paystack Provider ==============

pub struct PaystackProvider {
    secret_key: String,
    http_client: reqwest::Client,
}

impl PaystackProvider {
    pub fn new(secret_key: String) -> Self {
        Self { secret_key, http_client: reqwest::Client::new() }
    }
}

#[async_trait]
impl PaymentProvider for PaystackProvider {
    fn provider_id(&self) -> &'static str { "paystack" }

    async fn initialize(&self, request: &PaymentRequest) -> Result<PaymentResponse, ProviderError> {
        let amount_kobo = (request.amount * Decimal::from(100)).to_string().parse::<i64>().unwrap_or(0);
        
        let payload = json!({
            "email": request.email,
            "amount": amount_kobo,
            "currency": request.currency,
            "callback_url": request.callback_url,
            "metadata": request.metadata
        });

        let response = self.http_client
            .post("https://api.paystack.co/transaction/initialize")
            .bearer_auth(&self.secret_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        if result["status"].as_bool() != Some(true) {
            return Err(ProviderError::Api(result["message"].as_str().unwrap_or("Unknown error").to_string()));
        }

        Ok(PaymentResponse {
            reference: result["data"]["reference"].as_str().unwrap_or("").to_string(),
            authorization_url: result["data"]["authorization_url"].as_str().unwrap_or("").to_string(),
            access_code: result["data"]["access_code"].as_str().unwrap_or("").to_string(),
        })
    }

    async fn verify(&self, reference: &str) -> Result<VerificationResponse, ProviderError> {
        let response = self.http_client
            .get(format!("https://api.paystack.co/transaction/verify/{}", reference))
            .bearer_auth(&self.secret_key)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        Ok(VerificationResponse {
            status: result["data"]["status"].as_str().unwrap_or("unknown").to_string(),
            reference: reference.to_string(),
            amount: Decimal::from(result["data"]["amount"].as_i64().unwrap_or(0)) / Decimal::from(100),
            paid_at: None,
        })
    }

    async fn refund(&self, reference: &str, _amount: Decimal) -> Result<String, ProviderError> {
        let response = self.http_client
            .post("https://api.paystack.co/refund")
            .bearer_auth(&self.secret_key)
            .json(&json!({ "transaction": reference }))
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        Ok(result["data"]["id"].to_string())
    }
}

// ============== Flutterwave Provider ==============

pub struct FlutterwaveProvider {
    secret_key: String,
    http_client: reqwest::Client,
}

impl FlutterwaveProvider {
    pub fn new(secret_key: String) -> Self {
        Self { secret_key, http_client: reqwest::Client::new() }
    }
}

#[async_trait]
impl PaymentProvider for FlutterwaveProvider {
    fn provider_id(&self) -> &'static str { "flutterwave" }

    async fn initialize(&self, request: &PaymentRequest) -> Result<PaymentResponse, ProviderError> {
        let tx_ref = format!("BRIVAS-{}", uuid::Uuid::new_v4());
        
        let payload = json!({
            "tx_ref": tx_ref,
            "amount": request.amount.to_string(),
            "currency": request.currency,
            "redirect_url": request.callback_url,
            "customer": { "email": request.email },
            "meta": request.metadata
        });

        let response = self.http_client
            .post("https://api.flutterwave.com/v3/payments")
            .bearer_auth(&self.secret_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        Ok(PaymentResponse {
            reference: tx_ref,
            authorization_url: result["data"]["link"].as_str().unwrap_or("").to_string(),
            access_code: String::new(),
        })
    }

    async fn verify(&self, reference: &str) -> Result<VerificationResponse, ProviderError> {
        let response = self.http_client
            .get(format!("https://api.flutterwave.com/v3/transactions/verify_by_reference?tx_ref={}", reference))
            .bearer_auth(&self.secret_key)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        Ok(VerificationResponse {
            status: result["data"]["status"].as_str().unwrap_or("unknown").to_string(),
            reference: reference.to_string(),
            amount: result["data"]["amount"].as_f64().map(Decimal::try_from).transpose().ok().flatten().unwrap_or_default(),
            paid_at: None,
        })
    }

    async fn refund(&self, reference: &str, amount: Decimal) -> Result<String, ProviderError> {
        let response = self.http_client
            .post(format!("https://api.flutterwave.com/v3/transactions/{}/refund", reference))
            .bearer_auth(&self.secret_key)
            .json(&json!({ "amount": amount.to_string() }))
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let result: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        Ok(result["data"]["id"].to_string())
    }
}
