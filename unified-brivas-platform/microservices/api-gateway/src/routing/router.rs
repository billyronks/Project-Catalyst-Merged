//! Service Router
//!
//! Routes requests to internal microservices.

use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServiceRouter {
    routes: Arc<DashMap<String, ServiceEndpoint>>,
}

#[derive(Clone)]
pub struct ServiceEndpoint {
    pub service_name: String,
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
    pub health_check_path: String,
}

#[derive(Clone, Copy)]
pub enum Protocol {
    Http,
    Grpc,
    WebSocket,
}

impl ServiceRouter {
    pub fn new() -> Self {
        let routes = Arc::new(DashMap::new());
        
        // Register default services
        let default_services = [
            ("smsc", "smsc.brivas.svc.cluster.local", 8080, Protocol::Http),
            ("ussd", "ussd-gateway.brivas.svc.cluster.local", 8080, Protocol::Http),
            ("billing", "billing.brivas.svc.cluster.local", 8080, Protocol::Http),
            ("user", "user-service.brivas.svc.cluster.local", 8080, Protocol::Http),
            ("payment", "payment-service.brivas.svc.cluster.local", 8080, Protocol::Http),
            ("messaging", "unified-messaging.brivas.svc.cluster.local", 8080, Protocol::Http),
        ];
        
        for (name, host, port, protocol) in default_services {
            routes.insert(name.to_string(), ServiceEndpoint {
                service_name: name.to_string(),
                host: host.to_string(),
                port,
                protocol,
                health_check_path: "/health".to_string(),
            });
        }
        
        Self { routes }
    }

    pub fn route(&self, path: &str) -> Option<ServiceEndpoint> {
        let service = path.split('/').nth(1)?;
        self.routes.get(service).map(|e| e.clone())
    }

    pub fn register(&self, name: &str, endpoint: ServiceEndpoint) {
        self.routes.insert(name.to_string(), endpoint);
    }

    pub fn list_services(&self) -> Vec<String> {
        self.routes.iter().map(|r| r.key().clone()).collect()
    }
}

impl Default for ServiceRouter {
    fn default() -> Self {
        Self::new()
    }
}
