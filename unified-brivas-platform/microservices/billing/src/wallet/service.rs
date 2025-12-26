//! Wallet Service
//!
//! Prepaid wallet management with real-time balance tracking.

use chrono::Utc;
use dashmap::DashMap;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::{TransactionType, Wallet, WalletTransaction};

#[derive(Clone)]
pub struct WalletService {
    /// Wallet storage
    wallets: Arc<DashMap<Uuid, Wallet>>,
    /// Transaction history
    transactions: Arc<DashMap<Uuid, Vec<WalletTransaction>>>,
    /// Low balance threshold
    low_balance_threshold: Decimal,
    /// LumaDB URL
    #[allow(dead_code)]
    lumadb_url: String,
}

impl WalletService {
    pub async fn new(lumadb_url: &str, low_balance_threshold: Decimal) -> brivas_core::Result<Self> {
        Ok(Self {
            wallets: Arc::new(DashMap::new()),
            transactions: Arc::new(DashMap::new()),
            low_balance_threshold,
            lumadb_url: lumadb_url.to_string(),
        })
    }

    /// Create a new wallet
    pub async fn create_wallet(
        &self,
        customer_id: Uuid,
        currency: &str,
        initial_balance: Decimal,
    ) -> brivas_core::Result<Wallet> {
        let wallet = Wallet {
            id: Uuid::new_v4(),
            customer_id,
            balance: initial_balance,
            currency: currency.to_string(),
            credit_limit: Decimal::ZERO,
            low_balance_alert: true,
            auto_topup_enabled: false,
            auto_topup_amount: None,
            auto_topup_threshold: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.wallets.insert(wallet.id, wallet.clone());
        self.transactions.insert(wallet.id, Vec::new());

        if initial_balance > Decimal::ZERO {
            self.record_transaction(
                wallet.id,
                TransactionType::Credit,
                initial_balance,
                Decimal::ZERO,
                initial_balance,
                "Initial deposit",
            ).await?;
        }

        Ok(wallet)
    }

    /// Get wallet by ID
    pub async fn get_wallet(&self, wallet_id: Uuid) -> Option<Wallet> {
        self.wallets.get(&wallet_id).map(|w| w.clone())
    }

    /// Get wallet by customer ID
    pub async fn get_by_customer(&self, customer_id: Uuid) -> Option<Wallet> {
        self.wallets
            .iter()
            .find(|w| w.value().customer_id == customer_id)
            .map(|w| w.value().clone())
    }

    /// Check balance
    pub async fn check_balance(&self, wallet_id: Uuid) -> brivas_core::Result<Decimal> {
        self.wallets
            .get(&wallet_id)
            .map(|w| w.balance)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Wallet not found").into())
    }

    /// Credit wallet (add funds)
    pub async fn credit(
        &self,
        wallet_id: Uuid,
        amount: Decimal,
        description: &str,
    ) -> brivas_core::Result<Decimal> {
        let mut wallet = self.wallets
            .get_mut(&wallet_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Wallet not found"))?;

        let balance_before = wallet.balance;
        wallet.balance += amount;
        wallet.updated_at = Utc::now();
        let balance_after = wallet.balance;

        drop(wallet);

        self.record_transaction(
            wallet_id,
            TransactionType::Credit,
            amount,
            balance_before,
            balance_after,
            description,
        ).await?;

        Ok(balance_after)
    }

    /// Debit wallet (deduct funds)
    pub async fn debit(
        &self,
        wallet_id: Uuid,
        amount: Decimal,
        description: &str,
    ) -> brivas_core::Result<Decimal> {
        let mut wallet = self.wallets
            .get_mut(&wallet_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Wallet not found"))?;

        let effective_balance = wallet.balance + wallet.credit_limit;
        if amount > effective_balance {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Insufficient funds",
            ).into());
        }

        let balance_before = wallet.balance;
        wallet.balance -= amount;
        wallet.updated_at = Utc::now();
        let balance_after = wallet.balance;

        // Check low balance alert
        if wallet.low_balance_alert && wallet.balance < self.low_balance_threshold {
            tracing::warn!(
                wallet_id = %wallet_id,
                balance = %wallet.balance,
                "Low balance alert"
            );
            // TODO: Send notification
        }

        drop(wallet);

        self.record_transaction(
            wallet_id,
            TransactionType::Debit,
            amount,
            balance_before,
            balance_after,
            description,
        ).await?;

        Ok(balance_after)
    }

    /// Has sufficient balance
    pub async fn has_sufficient_balance(&self, wallet_id: Uuid, amount: Decimal) -> bool {
        self.wallets
            .get(&wallet_id)
            .map(|w| w.balance + w.credit_limit >= amount)
            .unwrap_or(false)
    }

    /// Record transaction
    async fn record_transaction(
        &self,
        wallet_id: Uuid,
        transaction_type: TransactionType,
        amount: Decimal,
        balance_before: Decimal,
        balance_after: Decimal,
        description: &str,
    ) -> brivas_core::Result<()> {
        let tx = WalletTransaction {
            id: Uuid::new_v4(),
            wallet_id,
            transaction_type,
            amount,
            balance_before,
            balance_after,
            reference_id: None,
            description: description.to_string(),
            created_at: Utc::now(),
        };

        self.transactions
            .entry(wallet_id)
            .or_insert_with(Vec::new)
            .push(tx);

        Ok(())
    }

    /// Get transaction history
    pub async fn get_transactions(&self, wallet_id: Uuid, limit: usize) -> Vec<WalletTransaction> {
        self.transactions
            .get(&wallet_id)
            .map(|txs| txs.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default()
    }
}
