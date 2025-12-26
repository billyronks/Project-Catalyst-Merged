//! Billing Types

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Call Detail Record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cdr {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub service_type: ServiceType,
    pub source: String,
    pub destination: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: u32,
    pub quantity: u32,
    pub status: CdrStatus,
    pub rated_amount: Option<Decimal>,
    pub currency: String,
    pub rate_id: Option<Uuid>,
    pub carrier_id: Option<Uuid>,
    pub pop_id: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    Sms,
    SmsInternational,
    Ussd,
    Voice,
    VoiceInternational,
    Data,
    Rcs,
    WhatsApp,
    Telegram,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CdrStatus {
    Pending,
    Rated,
    Billed,
    Failed,
    Disputed,
}

/// Rate definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rate {
    pub id: Uuid,
    pub name: String,
    pub service_type: ServiceType,
    pub destination_pattern: String,
    pub unit_price: Decimal,
    pub currency: String,
    pub unit_type: UnitType,
    pub minimum_charge: Decimal,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    PerMessage,
    PerSecond,
    PerMinute,
    PerSession,
    PerMb,
}

/// Invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub invoice_number: String,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub currency: String,
    pub status: InvoiceStatus,
    pub due_date: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub line_items: Vec<InvoiceLineItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Draft,
    Pending,
    Sent,
    Paid,
    Overdue,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub id: Uuid,
    pub description: String,
    pub service_type: ServiceType,
    pub quantity: u32,
    pub unit_price: Decimal,
    pub amount: Decimal,
}

/// Prepaid Wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub balance: Decimal,
    pub currency: String,
    pub credit_limit: Decimal,
    pub low_balance_alert: bool,
    pub auto_topup_enabled: bool,
    pub auto_topup_amount: Option<Decimal>,
    pub auto_topup_threshold: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Wallet Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: Decimal,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub reference_id: Option<Uuid>,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Credit,
    Debit,
    Refund,
    Adjustment,
}
