//! Carrier management module
//!
//! Handles carrier CRUD operations, failover logic, and carrier caching.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::{Error, Result};

/// Carrier status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CarrierStatus {
    Active,
    Inactive,
    Maintenance,
    Suspended,
}

/// Carrier authentication type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    Digest,
    IpAcl,
    Both,
}

/// Carrier entity representing a VoIP carrier/gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Carrier {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub transport: String,
    pub status: CarrierStatus,
    pub auth_type: AuthType,
    pub username: Option<String>,
    // Password is never serialized
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub allowed_ips: Vec<String>,
    pub max_channels: i32,
    pub current_channels: i32,
    pub priority: i32,
    pub weight: i32,
    pub failover_carrier_id: Option<Uuid>,
    pub prefix: Option<String>,
    pub strip_digits: i32,
    pub prepend: Option<String>,
    pub codecs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new carrier
#[derive(Debug, Deserialize)]
pub struct CreateCarrierRequest {
    pub name: String,
    pub host: String,
    pub port: Option<u16>,
    pub transport: Option<String>,
    pub auth_type: Option<AuthType>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub allowed_ips: Option<Vec<String>>,
    pub max_channels: Option<i32>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub failover_carrier_id: Option<Uuid>,
    pub prefix: Option<String>,
    pub strip_digits: Option<i32>,
    pub prepend: Option<String>,
    pub codecs: Option<Vec<String>>,
}

/// Request to update a carrier
#[derive(Debug, Deserialize)]
pub struct UpdateCarrierRequest {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub transport: Option<String>,
    pub status: Option<CarrierStatus>,
    pub auth_type: Option<AuthType>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub allowed_ips: Option<Vec<String>>,
    pub max_channels: Option<i32>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub failover_carrier_id: Option<Uuid>,
    pub prefix: Option<String>,
    pub strip_digits: Option<i32>,
    pub prepend: Option<String>,
    pub codecs: Option<Vec<String>>,
}

/// Carrier statistics
#[derive(Debug, Clone, Serialize)]
pub struct CarrierStats {
    pub carrier_id: Uuid,
    pub carrier_name: String,
    pub total_calls: i64,
    pub successful_calls: i64,
    pub failed_calls: i64,
    pub asr: f64, // Answer-Seizure Ratio
    pub acd: f64, // Average Call Duration (seconds)
    pub pdd: f64, // Post-Dial Delay (ms)
    pub ner: f64, // Network Effectiveness Ratio
    pub current_channels: i32,
    pub max_channels: i32,
}

/// Carrier summary for dashboard
#[derive(Debug, Serialize)]
pub struct CarrierSummary {
    pub total_carriers: i64,
    pub active_carriers: i64,
    pub inactive_carriers: i64,
    pub total_active_calls: i64,
    pub total_capacity: i64,
    pub overall_asr: f64,
    pub overall_acd: f64,
}

/// Thread-safe carrier cache with TTL
pub struct CarrierCache {
    carriers: DashMap<Uuid, (Carrier, Instant)>,
    ttl: Duration,
}

impl CarrierCache {
    pub fn new() -> Self {
        Self {
            carriers: DashMap::new(),
            ttl: Duration::from_secs(300), // 5 minutes default
        }
    }

    pub fn with_ttl(ttl_secs: u64) -> Self {
        Self {
            carriers: DashMap::new(),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Get carrier from cache if not expired
    pub fn get(&self, id: &Uuid) -> Option<Carrier> {
        self.carriers.get(id).and_then(|entry| {
            if entry.1.elapsed() < self.ttl {
                Some(entry.0.clone())
            } else {
                None
            }
        })
    }

    /// Insert carrier into cache
    pub fn insert(&self, carrier: Carrier) {
        self.carriers.insert(carrier.id, (carrier, Instant::now()));
    }

    /// Remove carrier from cache
    pub fn remove(&self, id: &Uuid) {
        self.carriers.remove(id);
    }

    /// Clear all expired entries
    pub fn prune(&self) {
        self.carriers.retain(|_, (_, inserted)| inserted.elapsed() < self.ttl);
    }

    /// Get all active carriers from cache
    pub fn get_active_carriers(&self) -> Vec<Carrier> {
        self.carriers
            .iter()
            .filter(|entry| entry.1.elapsed() < self.ttl && entry.0.status == CarrierStatus::Active)
            .map(|entry| entry.0.clone())
            .collect()
    }
}

impl Default for CarrierCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Carrier repository for database operations
pub struct CarrierRepository<'a> {
    db: &'a brivas_lumadb::LumaDbPool,
}

impl<'a> CarrierRepository<'a> {
    pub fn new(db: &'a brivas_lumadb::LumaDbPool) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Carrier>> {
        let client = self.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;
        
        let row = client
            .query_opt(
                "SELECT * FROM carriers WHERE id = $1",
                &[&id],
            )
            .await?;

        Ok(row.map(|r| self.row_to_carrier(&r)))
    }

    pub async fn find_all(&self) -> Result<Vec<Carrier>> {
        let client = self.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;
        
        let rows = client
            .query("SELECT * FROM carriers ORDER BY priority ASC, name ASC", &[])
            .await?;

        Ok(rows.iter().map(|r| self.row_to_carrier(r)).collect())
    }

    pub async fn find_active(&self) -> Result<Vec<Carrier>> {
        let client = self.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;
        
        let rows = client
            .query(
                "SELECT * FROM carriers WHERE status = 'active' ORDER BY priority ASC",
                &[],
            )
            .await?;

        Ok(rows.iter().map(|r| self.row_to_carrier(r)).collect())
    }

    pub async fn create(&self, req: CreateCarrierRequest) -> Result<Carrier> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let client = self.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

        client
            .execute(
                r#"
                INSERT INTO carriers (
                    id, name, host, port, transport, status, auth_type,
                    username, password, allowed_ips, max_channels, current_channels,
                    priority, weight, failover_carrier_id, prefix, strip_digits,
                    prepend, codecs, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, 'active', $6,
                    $7, $8, $9, $10, 0,
                    $11, $12, $13, $14, $15,
                    $16, $17, $18, $18
                )
                "#,
                &[
                    &id,
                    &req.name,
                    &req.host,
                    &(req.port.unwrap_or(5060) as i32),
                    &req.transport.unwrap_or_else(|| "udp".to_string()),
                    &serde_json::to_string(&req.auth_type.unwrap_or(AuthType::IpAcl)).unwrap(),
                    &req.username,
                    &req.password,
                    &req.allowed_ips.unwrap_or_default(),
                    &req.max_channels.unwrap_or(100),
                    &req.priority.unwrap_or(1),
                    &req.weight.unwrap_or(10),
                    &req.failover_carrier_id,
                    &req.prefix,
                    &req.strip_digits.unwrap_or(0),
                    &req.prepend,
                    &req.codecs.unwrap_or_else(|| vec!["g711u".to_string(), "g711a".to_string()]),
                    &now,
                ],
            )
            .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| Error::Internal("Failed to create carrier".to_string()))
    }

    pub async fn update(&self, id: Uuid, req: UpdateCarrierRequest) -> Result<Carrier> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| Error::CarrierNotFound(id.to_string()))?;

        let client = self.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;
        let now = Utc::now();

        client
            .execute(
                r#"
                UPDATE carriers SET
                    name = COALESCE($2, name),
                    host = COALESCE($3, host),
                    port = COALESCE($4, port),
                    transport = COALESCE($5, transport),
                    status = COALESCE($6, status),
                    updated_at = $7
                WHERE id = $1
                "#,
                &[
                    &id,
                    &req.name,
                    &req.host,
                    &req.port.map(|p| p as i32),
                    &req.transport,
                    &req.status.map(|s| format!("{:?}", s).to_lowercase()),
                    &now,
                ],
            )
            .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| Error::Internal("Failed to update carrier".to_string()))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let client = self.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;
        
        let result = client
            .execute("DELETE FROM carriers WHERE id = $1", &[&id])
            .await?;

        if result == 0 {
            return Err(Error::CarrierNotFound(id.to_string()));
        }

        Ok(())
    }

    fn row_to_carrier(&self, row: &tokio_postgres::Row) -> Carrier {
        Carrier {
            id: row.get("id"),
            name: row.get("name"),
            host: row.get("host"),
            port: row.get::<_, i32>("port") as u16,
            transport: row.get("transport"),
            status: serde_json::from_str(row.get("status")).unwrap_or(CarrierStatus::Active),
            auth_type: serde_json::from_str(row.get("auth_type")).unwrap_or(AuthType::IpAcl),
            username: row.get("username"),
            password: row.get("password"),
            allowed_ips: row.get("allowed_ips"),
            max_channels: row.get("max_channels"),
            current_channels: row.get("current_channels"),
            priority: row.get("priority"),
            weight: row.get("weight"),
            failover_carrier_id: row.get("failover_carrier_id"),
            prefix: row.get("prefix"),
            strip_digits: row.get("strip_digits"),
            prepend: row.get("prepend"),
            codecs: row.get("codecs"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
