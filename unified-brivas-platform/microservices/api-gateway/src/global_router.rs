//! Global Router Module - Multi-protocol routing


/// Route information for API requests
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub service: String,
    pub method: String,
    pub path: String,
}

/// Global router for multi-protocol requests
pub struct GlobalRouter {
    routes: Vec<RouteInfo>,
}

impl GlobalRouter {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route(&mut self, route: RouteInfo) {
        self.routes.push(route);
    }

    pub fn find_route(&self, path: &str) -> Option<&RouteInfo> {
        self.routes.iter().find(|r| r.path == path)
    }
}

impl Default for GlobalRouter {
    fn default() -> Self {
        Self::new()
    }
}
