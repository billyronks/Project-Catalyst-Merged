//! Intelligent message routing engine

use brivas_core::{Operator, PhoneNumber, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Message routing engine with cost/quality optimization
#[derive(Clone)]
pub struct MessageRouter {
    routes: Arc<RwLock<RoutingTable>>,
    #[allow(dead_code)]
    db_url: String,
}

/// Route definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub id: String,
    pub carrier_id: String,
    pub connection_id: String,
    pub operator: Operator,
    pub priority: u8,
    pub cost_per_message: Decimal,
    pub quality_score: f64,
    pub active: bool,
    pub features: RouteFeatures,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteFeatures {
    pub supports_unicode: bool,
    pub supports_flash: bool,
    pub supports_concatenation: bool,
    pub max_segments: u8,
    pub delivery_report: bool,
}

/// Routing criteria for route selection
#[derive(Debug, Clone)]
pub struct RoutingCriteria {
    pub cost_weight: f64,
    pub quality_weight: f64,
    pub require_dlr: bool,
    pub require_unicode: bool,
}

impl Default for RoutingCriteria {
    fn default() -> Self {
        Self {
            cost_weight: 0.3,
            quality_weight: 0.7,
            require_dlr: true,
            require_unicode: false,
        }
    }
}

struct RoutingTable {
    routes: HashMap<Operator, Vec<Route>>,
}

impl RoutingTable {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    fn get_routes(&self, operator: &Operator) -> Vec<Route> {
        self.routes.get(operator).cloned().unwrap_or_default()
    }

    fn add_route(&mut self, route: Route) {
        self.routes
            .entry(route.operator)
            .or_default()
            .push(route);
    }
}

impl MessageRouter {
    pub async fn new(db_url: &str) -> Result<Self> {
        let mut table = RoutingTable::new();

        // Initialize with default routes for Nigerian operators
        table.add_route(Route {
            id: "mtn-primary".to_string(),
            carrier_id: "mtn-ng".to_string(),
            connection_id: "smpp-mtn-1".to_string(),
            operator: Operator::Mtn,
            priority: 1,
            cost_per_message: Decimal::new(250, 2), // 2.50
            quality_score: 0.95,
            active: true,
            features: RouteFeatures {
                supports_unicode: true,
                supports_flash: true,
                supports_concatenation: true,
                max_segments: 10,
                delivery_report: true,
            },
        });

        table.add_route(Route {
            id: "airtel-primary".to_string(),
            carrier_id: "airtel-ng".to_string(),
            connection_id: "smpp-airtel-1".to_string(),
            operator: Operator::Airtel,
            priority: 1,
            cost_per_message: Decimal::new(250, 2),
            quality_score: 0.92,
            active: true,
            features: RouteFeatures {
                supports_unicode: true,
                supports_flash: false,
                supports_concatenation: true,
                max_segments: 8,
                delivery_report: true,
            },
        });

        table.add_route(Route {
            id: "glo-primary".to_string(),
            carrier_id: "glo-ng".to_string(),
            connection_id: "smpp-glo-1".to_string(),
            operator: Operator::Glo,
            priority: 1,
            cost_per_message: Decimal::new(200, 2), // 2.00
            quality_score: 0.88,
            active: true,
            features: RouteFeatures {
                supports_unicode: true,
                supports_flash: false,
                supports_concatenation: true,
                max_segments: 6,
                delivery_report: true,
            },
        });

        table.add_route(Route {
            id: "9mobile-primary".to_string(),
            carrier_id: "9mobile-ng".to_string(),
            connection_id: "smpp-9mobile-1".to_string(),
            operator: Operator::NineMobile,
            priority: 1,
            cost_per_message: Decimal::new(220, 2), // 2.20
            quality_score: 0.85,
            active: true,
            features: RouteFeatures {
                supports_unicode: true,
                supports_flash: false,
                supports_concatenation: true,
                max_segments: 5,
                delivery_report: true,
            },
        });

        Ok(Self {
            routes: Arc::new(RwLock::new(table)),
            db_url: db_url.to_string(),
        })
    }

    /// Find the best route for a message
    pub async fn find_best_route(
        &self,
        destination: &str,
        criteria: &RoutingCriteria,
    ) -> Result<Option<Route>> {
        let phone = PhoneNumber::new(destination);
        let operator = phone.operator();

        let table = self.routes.read().await;
        let routes = table.get_routes(&operator);

        if routes.is_empty() {
            return Ok(None);
        }

        // Filter eligible routes
        let eligible: Vec<_> = routes
            .into_iter()
            .filter(|r| {
                r.active
                    && (!criteria.require_dlr || r.features.delivery_report)
                    && (!criteria.require_unicode || r.features.supports_unicode)
            })
            .collect();

        if eligible.is_empty() {
            return Ok(None);
        }

        // Score and select best
        let best = eligible
            .into_iter()
            .max_by(|a, b| {
                let score_a = self.score_route(a, criteria);
                let score_b = self.score_route(b, criteria);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        Ok(best)
    }

    fn score_route(&self, route: &Route, criteria: &RoutingCriteria) -> f64 {
        // Normalize cost score (lower is better)
        let cost_score = 1.0 / (1.0 + route.cost_per_message.to_string().parse::<f64>().unwrap_or(1.0));

        // Combine weighted scores
        criteria.cost_weight * cost_score + criteria.quality_weight * route.quality_score
    }

    /// Add a new route
    pub async fn add_route(&self, route: Route) -> Result<()> {
        let mut table = self.routes.write().await;
        table.add_route(route);
        Ok(())
    }

    /// Get all routes
    pub async fn list_routes(&self) -> Vec<Route> {
        let table = self.routes.read().await;
        table
            .routes
            .values()
            .flatten()
            .cloned()
            .collect()
    }
}
