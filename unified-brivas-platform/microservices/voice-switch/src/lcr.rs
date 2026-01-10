//! Least Cost Routing (LCR) Engine
//!
//! Implements intelligent call routing based on:
//! - Cost optimization
//! - Quality metrics (ASR, PDD)
//! - Carrier capacity and availability
//! - Time-based routing rules

use std::sync::Arc;
use uuid::Uuid;

use crate::carrier::{Carrier, CarrierCache, CarrierStatus};
use crate::kdb::KdbClient;
use crate::{Error, Result};

/// LCR routing decision
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoutingDecision {
    pub call_id: Uuid,
    pub destination: String,
    pub selected_carrier: RoutedCarrier,
    pub fallback_carriers: Vec<RoutedCarrier>,
    pub routing_reason: String,
    pub estimated_cost: f64,
    pub estimated_quality: f64,
}

/// Carrier selected for routing
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoutedCarrier {
    pub carrier_id: Uuid,
    pub carrier_name: String,
    pub host: String,
    pub port: u16,
    pub transport: String,
    pub dial_string: String,
    pub strip_digits: i32,
    pub prepend: Option<String>,
    pub priority: i32,
    pub estimated_asr: f64,
    pub rate: f64,
}

/// Routing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMode {
    /// Cheapest route first
    LeastCost,
    /// Best quality (ASR/PDD) first
    Quality,
    /// Balance between cost and quality
    Balanced,
    /// Follow priority ordering
    Priority,
    /// Round-robin for load balancing
    RoundRobin,
}

/// LCR Engine for intelligent call routing
pub struct LcrEngine {
    carrier_cache: Arc<CarrierCache>,
    kdb_client: Arc<KdbClient>,
    db: brivas_lumadb::LumaDbPool,
}

impl LcrEngine {
    pub fn new(
        carrier_cache: Arc<CarrierCache>,
        kdb_client: Arc<KdbClient>,
        db: brivas_lumadb::LumaDbPool,
    ) -> Self {
        Self {
            carrier_cache,
            kdb_client,
            db,
        }
    }

    /// Find best route for a destination number
    pub async fn route(
        &self,
        destination: &str,
        mode: RoutingMode,
    ) -> Result<RoutingDecision> {
        let call_id = Uuid::new_v4();

        // Get matching routes from database
        let routes = self.find_matching_routes(destination).await?;
        if routes.is_empty() {
            return Err(Error::NoRouteAvailable(destination.to_string()));
        }

        // Get carrier stats from kdb+ for quality-aware routing
        let carrier_stats = self.kdb_client.get_carrier_stats(None).await.ok();

        // Score and sort carriers based on routing mode
        let mut scored_carriers: Vec<(Carrier, f64, f64)> = Vec::new();

        for route in &routes {
            // Get carrier (from cache or database)
            let carrier = self.get_carrier(route.carrier_id).await?;

            // Skip inactive carriers
            if carrier.status != CarrierStatus::Active {
                continue;
            }

            // Skip carriers at capacity
            if carrier.current_channels >= carrier.max_channels {
                continue;
            }

            // Get carrier quality metrics
            let (asr, rate) = if let Some(ref stats) = carrier_stats {
                stats
                    .iter()
                    .find(|s| s.carrier_id == carrier.id)
                    .map(|s| (s.asr, route.rate))
                    .unwrap_or((90.0, route.rate))
            } else {
                (90.0, route.rate) // Default values
            };

            scored_carriers.push((carrier, rate, asr));
        }

        if scored_carriers.is_empty() {
            return Err(Error::NoRouteAvailable(destination.to_string()));
        }

        // Sort based on routing mode
        self.sort_carriers(&mut scored_carriers, mode);

        // Build routing decision
        let (primary, rate, asr) = scored_carriers.remove(0);
        let dial_string = self.build_dial_string(&primary, destination);

        let selected_carrier = RoutedCarrier {
            carrier_id: primary.id,
            carrier_name: primary.name.clone(),
            host: primary.host.clone(),
            port: primary.port,
            transport: primary.transport.clone(),
            dial_string,
            strip_digits: primary.strip_digits,
            prepend: primary.prepend.clone(),
            priority: primary.priority,
            estimated_asr: asr,
            rate,
        };

        // Build fallback list
        let fallback_carriers: Vec<RoutedCarrier> = scored_carriers
            .into_iter()
            .take(3) // Max 3 fallbacks
            .map(|(carrier, rate, asr)| {
                let dial_string = self.build_dial_string(&carrier, destination);
                RoutedCarrier {
                    carrier_id: carrier.id,
                    carrier_name: carrier.name,
                    host: carrier.host,
                    port: carrier.port,
                    transport: carrier.transport,
                    dial_string,
                    strip_digits: carrier.strip_digits,
                    prepend: carrier.prepend,
                    priority: carrier.priority,
                    estimated_asr: asr,
                    rate,
                }
            })
            .collect();

        let routing_reason = match mode {
            RoutingMode::LeastCost => "Lowest cost route".to_string(),
            RoutingMode::Quality => "Best quality (ASR) route".to_string(),
            RoutingMode::Balanced => "Balanced cost/quality route".to_string(),
            RoutingMode::Priority => "Priority-based routing".to_string(),
            RoutingMode::RoundRobin => "Load-balanced routing".to_string(),
        };

        Ok(RoutingDecision {
            call_id,
            destination: destination.to_string(),
            selected_carrier,
            fallback_carriers,
            routing_reason,
            estimated_cost: rate,
            estimated_quality: asr,
        })
    }

    /// Find routes matching a destination prefix
    async fn find_matching_routes(&self, destination: &str) -> Result<Vec<Route>> {
        let client = self.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

        // Find longest prefix match
        let rows = client
            .query(
                r#"
                SELECT r.id, r.carrier_id, r.prefix, r.rate, r.priority, r.enabled
                FROM routes r
                WHERE $1 LIKE (r.prefix || '%')
                  AND r.enabled = true
                ORDER BY LENGTH(r.prefix) DESC, r.priority ASC
                "#,
                &[&destination],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|row| Route {
                id: row.get("id"),
                carrier_id: row.get("carrier_id"),
                prefix: row.get("prefix"),
                rate: row.get("rate"),
                priority: row.get("priority"),
            })
            .collect())
    }

    /// Get carrier by ID (cache-first)
    async fn get_carrier(&self, id: Uuid) -> Result<Carrier> {
        // Try cache first
        if let Some(carrier) = self.carrier_cache.get(&id) {
            return Ok(carrier);
        }

        // Fetch from database
        let client = self.db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

        let row = client
            .query_opt("SELECT * FROM carriers WHERE id = $1", &[&id])
            .await?
            .ok_or_else(|| Error::CarrierNotFound(id.to_string()))?;

        // Parse carrier from row (simplified)
        let carrier = Carrier {
            id: row.get("id"),
            name: row.get("name"),
            host: row.get("host"),
            port: row.get::<_, i32>("port") as u16,
            transport: row.get("transport"),
            status: CarrierStatus::Active,
            auth_type: crate::carrier::AuthType::IpAcl,
            username: row.get("username"),
            password: None,
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
        };

        // Update cache
        self.carrier_cache.insert(carrier.clone());

        Ok(carrier)
    }

    /// Sort carriers based on routing mode
    fn sort_carriers(&self, carriers: &mut [(Carrier, f64, f64)], mode: RoutingMode) {
        match mode {
            RoutingMode::LeastCost => {
                carriers.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            }
            RoutingMode::Quality => {
                carriers.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap()); // Higher ASR first
            }
            RoutingMode::Balanced => {
                // Score = ASR * (1 - normalized_cost)
                carriers.sort_by(|a, b| {
                    let max_rate = carriers.iter().map(|c| c.1).fold(0.0f64, f64::max);
                    let score_a = a.2 * (1.0 - a.1 / max_rate);
                    let score_b = b.2 * (1.0 - b.1 / max_rate);
                    score_b.partial_cmp(&score_a).unwrap()
                });
            }
            RoutingMode::Priority => {
                carriers.sort_by(|a, b| a.0.priority.cmp(&b.0.priority));
            }
            RoutingMode::RoundRobin => {
                // Shuffle for load distribution
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                carriers.shuffle(&mut rng);
            }
        }
    }

    /// Build SIP dial string for carrier
    fn build_dial_string(&self, carrier: &Carrier, destination: &str) -> String {
        let mut number = destination.to_string();

        // Strip digits
        if carrier.strip_digits > 0 {
            number = number[carrier.strip_digits as usize..].to_string();
        }

        // Prepend
        if let Some(ref prepend) = carrier.prepend {
            number = format!("{}{}", prepend, number);
        }

        // Build SIP URI
        format!(
            "sip:{}@{}:{}",
            number, carrier.host, carrier.port
        )
    }
}

/// Route entity
#[derive(Debug, Clone)]
struct Route {
    id: Uuid,
    carrier_id: Uuid,
    prefix: String,
    rate: f64,
    priority: i32,
}
