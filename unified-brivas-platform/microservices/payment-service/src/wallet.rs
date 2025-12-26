//! Wallet management service

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub account_id: String,
    pub balance: Decimal,
    pub currency: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct WalletService {
    #[allow(dead_code)]
    db_url: String,
}

impl WalletService {
    pub async fn new(db_url: &str) -> brivas_core::Result<Self> {
        Ok(Self { db_url: db_url.to_string() })
    }

    pub async fn get_balance(&self, account_id: &str) -> brivas_core::Result<Decimal> {
        tracing::debug!(account_id = %account_id, "Getting wallet balance");
        Ok(Decimal::ZERO)
    }

    pub async fn credit(&self, account_id: &str, amount: Decimal, reference: &str) -> brivas_core::Result<Decimal> {
        tracing::info!(account_id = %account_id, amount = %amount, reference = %reference, "Crediting wallet");
        Ok(amount)
    }

    pub async fn debit(&self, account_id: &str, amount: Decimal, reference: &str) -> brivas_core::Result<Decimal> {
        tracing::info!(account_id = %account_id, amount = %amount, reference = %reference, "Debiting wallet");
        Ok(Decimal::ZERO)
    }

    pub async fn transfer(&self, from: &str, to: &str, amount: Decimal) -> brivas_core::Result<()> {
        tracing::info!(from = %from, to = %to, amount = %amount, "Wallet transfer");
        Ok(())
    }
}
