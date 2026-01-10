//! Service Discovery Client
//!
//! Enables cross-cluster communication by discovering service endpoints
//! via Consul, environment variables, or static configuration.

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Service endpoint configuration
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub name: String,
    pub url: String,
    pub health_path: String,
    pub healthy: bool,
}

/// Service discovery mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMode {
    /// Use environment variables (default, for single-cluster)
    Environment,
    /// Use Consul for service discovery
    Consul,
    /// Use DNS-based discovery (Kubernetes)
    Dns,
}

/// Service Discovery Client
pub struct ServiceDiscovery {
    mode: DiscoveryMode,
    consul_addr: Option<String>,
    services: Arc<RwLock<HashMap<String, Vec<ServiceEndpoint>>>>,
}

impl ServiceDiscovery {
    /// Create new service discovery client
    pub fn new() -> Self {
        let mode = if env::var("CONSUL_ENABLED")
            .unwrap_or_default()
            .to_lowercase()
            == "true"
        {
            DiscoveryMode::Consul
        } else {
            DiscoveryMode::Environment
        };

        let consul_addr = env::var("CONSUL_HTTP_ADDR").ok();

        info!(
            "Service discovery initialized: mode={:?}, consul={:?}",
            mode, consul_addr
        );

        Self {
            mode,
            consul_addr,
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with explicit mode
    pub fn with_mode(mode: DiscoveryMode) -> Self {
        Self {
            mode,
            consul_addr: env::var("CONSUL_HTTP_ADDR").ok(),
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get service URL by name
    pub async fn get_service_url(&self, service_name: &str) -> Option<String> {
        match self.mode {
            DiscoveryMode::Environment => self.get_from_env(service_name),
            DiscoveryMode::Consul => self.get_from_consul(service_name).await,
            DiscoveryMode::Dns => self.get_from_dns(service_name).await,
        }
    }

    /// Get all endpoints for a service (for load balancing)
    pub async fn get_service_endpoints(&self, service_name: &str) -> Vec<ServiceEndpoint> {
        let services = self.services.read().await;
        services.get(service_name).cloned().unwrap_or_default()
    }

    /// Get URL from environment variable
    fn get_from_env(&self, service_name: &str) -> Option<String> {
        // Convert service name to env var: voice-switch -> VOICE_SWITCH_URL
        let env_key = format!(
            "{}_URL",
            service_name.to_uppercase().replace('-', "_")
        );
        env::var(&env_key).ok().or_else(|| {
            // Fallback to well-known mappings
            match service_name {
                "lumadb" | "postgres" => Some(
                    env::var("LUMADB_URL")
                        .unwrap_or_else(|_| "postgres://brivas:brivas_secret@lumadb:5432/brivas".to_string()),
                ),
                "redis" => Some(
                    env::var("REDIS_URL")
                        .unwrap_or_else(|_| "redis://lumadb:6379".to_string()),
                ),
                "questdb" => Some(format!(
                    "postgres://{}:{}",
                    env::var("QUESTDB_HOST").unwrap_or_else(|_| "questdb".to_string()),
                    env::var("QUESTDB_PG_PORT").unwrap_or_else(|_| "8812".to_string())
                )),
                "clickhouse" => Some(
                    env::var("CLICKHOUSE_URL")
                        .unwrap_or_else(|_| "http://clickhouse:8123".to_string()),
                ),
                "temporal" => Some(format!(
                    "{}:{}",
                    env::var("TEMPORAL_HOST").unwrap_or_else(|_| "temporal".to_string()),
                    env::var("TEMPORAL_PORT").unwrap_or_else(|_| "7233".to_string())
                )),
                "nats" => Some(
                    env::var("NATS_URL")
                        .unwrap_or_else(|_| "nats://nats:4222".to_string()),
                ),
                _ => None,
            }
        })
    }

    /// Get URL from Consul
    async fn get_from_consul(&self, service_name: &str) -> Option<String> {
        let consul_addr = self.consul_addr.as_ref()?;

        let url = format!(
            "http://{}/v1/catalog/service/{}",
            consul_addr, service_name
        );

        match reqwest::get(&url).await {
            Ok(response) => {
                if let Ok(services) = response.json::<Vec<ConsulService>>().await {
                    if let Some(svc) = services.first() {
                        return Some(format!("{}:{}", svc.service_address, svc.service_port));
                    }
                }
                None
            }
            Err(e) => {
                warn!("Failed to query Consul for {}: {}", service_name, e);
                // Fallback to environment
                self.get_from_env(service_name)
            }
        }
    }

    /// Get URL from DNS (Kubernetes-style)
    async fn get_from_dns(&self, service_name: &str) -> Option<String> {
        // In Kubernetes, services are accessible via:
        // <service-name>.<namespace>.svc.cluster.local
        let namespace = env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| "default".to_string());
        
        // Check SRV records for port discovery
        Some(format!("{}.{}.svc.cluster.local", service_name, namespace))
    }

    /// Register current service with discovery system
    pub async fn register_service(
        &self,
        name: &str,
        addr: &str,
        port: u16,
        health_path: &str,
    ) -> anyhow::Result<()> {
        if self.mode != DiscoveryMode::Consul {
            return Ok(());
        }

        let consul_addr = self.consul_addr.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Consul address not configured"))?;

        let registration = ConsulRegistration {
            name: name.to_string(),
            id: format!("{}-{}", name, uuid::Uuid::new_v4()),
            address: addr.to_string(),
            port,
            check: ConsulCheck {
                http: format!("http://{}:{}{}", addr, port, health_path),
                interval: "10s".to_string(),
                timeout: "5s".to_string(),
            },
        };

        let client = reqwest::Client::new();
        let url = format!("http://{}/v1/agent/service/register", consul_addr);

        client
            .put(&url)
            .json(&registration)
            .send()
            .await?;

        info!("Registered service {} with Consul", name);
        Ok(())
    }

    /// Deregister service on shutdown
    pub async fn deregister_service(&self, service_id: &str) -> anyhow::Result<()> {
        if let Some(consul_addr) = &self.consul_addr {
            let url = format!(
                "http://{}/v1/agent/service/deregister/{}",
                consul_addr, service_id
            );
            reqwest::Client::new().put(&url).send().await?;
        }
        Ok(())
    }
}

impl Default for ServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, serde::Deserialize)]
struct ConsulService {
    #[serde(rename = "ServiceAddress")]
    service_address: String,
    #[serde(rename = "ServicePort")]
    service_port: u16,
}

#[derive(Debug, serde::Serialize)]
struct ConsulRegistration {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Address")]
    address: String,
    #[serde(rename = "Port")]
    port: u16,
    #[serde(rename = "Check")]
    check: ConsulCheck,
}

#[derive(Debug, serde::Serialize)]
struct ConsulCheck {
    #[serde(rename = "HTTP")]
    http: String,
    #[serde(rename = "Interval")]
    interval: String,
    #[serde(rename = "Timeout")]
    timeout: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_service_discovery() {
        std::env::set_var("VOICE_SWITCH_URL", "http://localhost:8095");
        let discovery = ServiceDiscovery::with_mode(DiscoveryMode::Environment);
        
        let url = discovery.get_from_env("voice-switch");
        assert_eq!(url, Some("http://localhost:8095".to_string()));
    }

    #[test]
    fn test_fallback_mappings() {
        let discovery = ServiceDiscovery::with_mode(DiscoveryMode::Environment);
        
        let url = discovery.get_from_env("questdb");
        assert!(url.is_some());
    }
}
