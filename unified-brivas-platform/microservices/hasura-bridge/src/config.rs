//! Hasura Bridge Configuration

use brivas_core::Result;

#[derive(Debug, Clone)]
pub struct HasuraConfig {
    pub http_bind: String,
    pub lumadb_url: String,
    pub admin_secret: Option<String>,
    pub enable_console: bool,
    pub cors_domain: String,
}

impl HasuraConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_bind: std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            admin_secret: std::env::var("HASURA_ADMIN_SECRET").ok(),
            enable_console: std::env::var("HASURA_ENABLE_CONSOLE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            cors_domain: std::env::var("CORS_DOMAIN").unwrap_or_else(|_| "*".to_string()),
        })
    }
}
