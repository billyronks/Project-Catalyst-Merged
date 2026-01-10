//! Temporal Activity Definitions
//!
//! Activities are the building blocks of workflows - atomic units of work
//! that can be retried independently.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Database pool shared across activities
pub type DbPool = Arc<brivas_lumadb::LumaDbPool>;

// ============================================
// Customer Activities
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInfo {
    pub customer_id: Uuid,
    pub name: String,
    pub balance: f64,
    pub status: String,
    pub plan_type: String,
}

/// Get customer information
pub async fn get_customer_info(db: &DbPool, customer_id: Uuid) -> anyhow::Result<CustomerInfo> {
    let client = db.get().await?;
    
    let row = client
        .query_opt(
            "SELECT id, name, balance, status, plan_type FROM customers WHERE id = $1",
            &[&customer_id],
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Customer not found"))?;

    Ok(CustomerInfo {
        customer_id: row.get("id"),
        name: row.get("name"),
        balance: row.get("balance"),
        status: row.get("status"),
        plan_type: row.get("plan_type"),
    })
}

/// Check if customer has sufficient balance
pub async fn check_balance(db: &DbPool, customer_id: Uuid, amount: f64) -> anyhow::Result<bool> {
    let info = get_customer_info(db, customer_id).await?;
    Ok(info.balance >= amount)
}

/// Debit customer account
pub async fn debit_account(
    db: &DbPool,
    customer_id: Uuid,
    amount: f64,
    description: &str,
) -> anyhow::Result<Uuid> {
    let transaction_id = Uuid::new_v4();
    let client = db.get().await?;

    client
        .execute(
            r#"
            UPDATE customers 
            SET balance = balance - $2, updated_at = NOW()
            WHERE id = $1 AND balance >= $2
            "#,
            &[&customer_id, &amount],
        )
        .await?;

    client
        .execute(
            r#"
            INSERT INTO transactions (id, customer_id, amount, type, description, created_at)
            VALUES ($1, $2, $3, 'debit', $4, NOW())
            "#,
            &[&transaction_id, &customer_id, &amount, &description],
        )
        .await?;

    Ok(transaction_id)
}

// ============================================
// Carrier Activities
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarrierRoute {
    pub carrier_id: Uuid,
    pub carrier_name: String,
    pub dial_string: String,
    pub rate: f64,
    pub available_channels: i32,
}

/// Find best route for destination
pub async fn find_route(
    db: &DbPool,
    destination: &str,
    routing_mode: &str,
) -> anyhow::Result<CarrierRoute> {
    let client = db.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT c.id, c.name, c.host, c.port, r.rate, 
                   (c.max_channels - c.current_channels) as available
            FROM routes r
            JOIN carriers c ON r.carrier_id = c.id
            WHERE $1 LIKE (r.prefix || '%')
              AND c.status = 'active'
              AND c.current_channels < c.max_channels
            ORDER BY r.rate ASC
            LIMIT 1
            "#,
            &[&destination],
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("No route available"))?;

    Ok(CarrierRoute {
        carrier_id: row.get("id"),
        carrier_name: row.get("name"),
        dial_string: format!(
            "sip:{}@{}:{}",
            destination,
            row.get::<_, String>("host"),
            row.get::<_, i32>("port")
        ),
        rate: row.get("rate"),
        available_channels: row.get("available"),
    })
}

/// Reserve a channel on carrier
pub async fn reserve_channel(db: &DbPool, carrier_id: Uuid) -> anyhow::Result<()> {
    let client = db.get().await?;

    let result = client
        .execute(
            r#"
            UPDATE carriers 
            SET current_channels = current_channels + 1
            WHERE id = $1 AND current_channels < max_channels
            "#,
            &[&carrier_id],
        )
        .await?;

    if result == 0 {
        return Err(anyhow::anyhow!("No available channels"));
    }

    Ok(())
}

/// Release a channel on carrier
pub async fn release_channel(db: &DbPool, carrier_id: Uuid) -> anyhow::Result<()> {
    let client = db.get().await?;

    client
        .execute(
            "UPDATE carriers SET current_channels = current_channels - 1 WHERE id = $1",
            &[&carrier_id],
        )
        .await?;

    Ok(())
}

// ============================================
// Service Provisioning Activities
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedResource {
    pub resource_id: Uuid,
    pub resource_type: String,
    pub value: String,
}

/// Allocate a DID number
pub async fn allocate_did(db: &DbPool, customer_id: Uuid, area_code: &str) -> anyhow::Result<AllocatedResource> {
    let client = db.get().await?;
    let did_id = Uuid::new_v4();

    // In production, this would actually allocate from a DID inventory
    let did_number = format!("+1{}5550{:04}", area_code, rand::random::<u16>() % 10000);

    client
        .execute(
            "INSERT INTO did_numbers (id, customer_id, number, status) VALUES ($1, $2, $3, 'active')",
            &[&did_id, &customer_id, &did_number],
        )
        .await?;

    Ok(AllocatedResource {
        resource_id: did_id,
        resource_type: "did".to_string(),
        value: did_number,
    })
}

/// Configure routing for a DID
pub async fn configure_routing(
    db: &DbPool,
    did_id: Uuid,
    destination: &str,
) -> anyhow::Result<()> {
    let client = db.get().await?;

    client
        .execute(
            "UPDATE did_numbers SET forward_to = $2, updated_at = NOW() WHERE id = $1",
            &[&did_id, &destination],
        )
        .await?;

    Ok(())
}

// ============================================
// Notification Activities
// ============================================

/// Send SMS notification
pub async fn send_sms(to: &str, message: &str) -> anyhow::Result<()> {
    tracing::info!("Sending SMS to {}: {}", to, message);
    // In production, this would call the SMSC microservice
    Ok(())
}

/// Send email notification
pub async fn send_email(to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
    tracing::info!("Sending email to {}: {}", to, subject);
    // In production, this would call an email service
    Ok(())
}

/// Send webhook notification
pub async fn send_webhook(url: &str, payload: serde_json::Value) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    client.post(url).json(&payload).send().await?;
    Ok(())
}

// ============================================
// CDR Activities
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdrRecord {
    pub call_id: Uuid,
    pub source: String,
    pub destination: String,
    pub carrier_id: Uuid,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub duration_secs: i64,
    pub disposition: String,
    pub cost: f64,
}

/// Write CDR to database
pub async fn write_cdr(db: &DbPool, cdr: CdrRecord) -> anyhow::Result<()> {
    let client = db.get().await?;

    client
        .execute(
            r#"
            INSERT INTO cdrs (
                call_id, source, destination, carrier_id,
                start_time, duration_secs, disposition, cost, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            "#,
            &[
                &cdr.call_id,
                &cdr.source,
                &cdr.destination,
                &cdr.carrier_id,
                &cdr.start_time,
                &cdr.duration_secs,
                &cdr.disposition,
                &cdr.cost,
            ],
        )
        .await?;

    Ok(())
}
