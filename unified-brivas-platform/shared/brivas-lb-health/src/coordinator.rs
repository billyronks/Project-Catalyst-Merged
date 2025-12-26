//! Global Load Balancer Coordinator
//!
//! Aggregates health from all PoPs for global routing decisions.

use std::collections::HashMap;
use chrono::Utc;

use crate::types::*;

/// Region proximity scores (0-1, higher = closer)
const REGION_PROXIMITY: &[(&str, &str, f64)] = &[
    ("africa-west", "africa-west", 1.0),
    ("africa-west", "africa-south", 0.7),
    ("africa-west", "africa-east", 0.6),
    ("africa-west", "europe-west", 0.5),
    ("africa-south", "africa-south", 1.0),
    ("africa-south", "africa-east", 0.7),
    ("europe-west", "europe-west", 1.0),
    ("europe-west", "europe-central", 0.9),
    ("europe-central", "europe-central", 1.0),
    ("asia-pacific", "asia-pacific", 1.0),
    ("asia-pacific", "asia-northeast", 0.8),
    ("asia-pacific", "asia-south", 0.7),
    ("south-america", "south-america", 1.0),
    // Default cross-region
];

/// Global Load Balancer Coordinator
pub struct GlobalLbCoordinator {
    #[allow(dead_code)]
    db_url: String,
    pop_health: HashMap<String, PopHealth>,
    pop_regions: HashMap<String, String>,
}

impl GlobalLbCoordinator {
    pub fn new(db_url: String) -> Self {
        // Initialize with known PoPs and regions
        let mut pop_regions = HashMap::new();
        pop_regions.insert("lagos-ng-1".to_string(), "africa-west".to_string());
        pop_regions.insert("johannesburg-za-1".to_string(), "africa-south".to_string());
        pop_regions.insert("london-uk-1".to_string(), "europe-west".to_string());
        pop_regions.insert("frankfurt-de-1".to_string(), "europe-central".to_string());
        pop_regions.insert("singapore-sg-1".to_string(), "asia-pacific".to_string());
        pop_regions.insert("saopaulo-br-1".to_string(), "south-america".to_string());
        pop_regions.insert("nairobi-ke-1".to_string(), "africa-east".to_string());
        pop_regions.insert("cairo-eg-1".to_string(), "africa-north".to_string());
        pop_regions.insert("dubai-ae-1".to_string(), "middle-east".to_string());
        pop_regions.insert("mumbai-in-1".to_string(), "asia-south".to_string());
        pop_regions.insert("tokyo-jp-1".to_string(), "asia-northeast".to_string());
        
        Self {
            db_url,
            pop_health: HashMap::new(),
            pop_regions,
        }
    }
    
    /// Update health for a PoP
    pub fn update_pop_health(&mut self, pop_id: String, health: PopHealth) {
        self.pop_health.insert(pop_id, health);
    }
    
    /// Get global view of all PoP health
    pub fn get_global_health(&self) -> GlobalHealthView {
        GlobalHealthView {
            pops: self.pop_health.clone(),
            updated_at: Utc::now(),
        }
    }
    
    /// Determine optimal PoP for a request
    pub fn get_optimal_pop(
        &self,
        client_region: &str,
        service: &str,
    ) -> Result<String, CoordinatorError> {
        // Filter to healthy PoPs that have the service
        let healthy_pops: Vec<(&String, &PopHealth)> = self.pop_health
            .iter()
            .filter(|(_, h)| h.services.iter().any(|s| s.service == service && s.healthy))
            .filter(|(_, h)| h.healthy)
            .collect();
        
        if healthy_pops.is_empty() {
            return Err(CoordinatorError::NoHealthyPop);
        }
        
        // Score by proximity and health
        let best_pop = healthy_pops
            .into_iter()
            .max_by(|(pop_a, health_a), (pop_b, health_b)| {
                let score_a = self.calculate_pop_score(client_region, pop_a, health_a);
                let score_b = self.calculate_pop_score(client_region, pop_b, health_b);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(pop_id, _)| pop_id.clone())
            .ok_or(CoordinatorError::NoHealthyPop)?;
        
        Ok(best_pop)
    }
    
    /// Calculate routing score for a PoP
    fn calculate_pop_score(&self, client_region: &str, pop_id: &str, health: &PopHealth) -> f64 {
        let proximity_score = self.get_region_proximity(client_region, pop_id);
        let health_score = health.overall_health_score();
        let capacity_score = health.available_capacity_score();
        
        // Weighted combination
        0.4 * proximity_score + 0.4 * health_score + 0.2 * capacity_score
    }
    
    /// Get proximity score between client region and PoP
    fn get_region_proximity(&self, client_region: &str, pop_id: &str) -> f64 {
        let pop_region = self.pop_regions
            .get(pop_id)
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        
        // Look up proximity
        REGION_PROXIMITY
            .iter()
            .find(|(r1, r2, _)| {
                (*r1 == client_region && *r2 == pop_region) ||
                (*r2 == client_region && *r1 == pop_region)
            })
            .map(|(_, _, score)| *score)
            .unwrap_or(0.3) // Default cross-region score
    }
    
    /// Get all PoPs that can serve a service
    pub fn get_pops_for_service(&self, service: &str) -> Vec<String> {
        self.pop_health
            .iter()
            .filter(|(_, h)| h.services.iter().any(|s| s.service == service))
            .map(|(id, _)| id.clone())
            .collect()
    }
}
