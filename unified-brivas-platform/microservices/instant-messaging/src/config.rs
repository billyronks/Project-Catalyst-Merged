//! IM Configuration

use brivas_core::Result;

#[derive(Debug, Clone)]
pub struct ImConfig {
    pub pop_id: String,
    pub http_bind: String,
    pub ws_bind: String,
    pub grpc_bind: String,
    pub lumadb_url: String,
    pub max_file_size_mb: u32,
    pub e2ee_enabled: bool,
}

impl ImConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            pop_id: std::env::var("POP_ID").unwrap_or_else(|_| "local".to_string()),
            http_bind: std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            ws_bind: std::env::var("WS_BIND").unwrap_or_else(|_| "0.0.0.0:8081".to_string()),
            grpc_bind: std::env::var("GRPC_BIND").unwrap_or_else(|_| "0.0.0.0:9090".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            max_file_size_mb: std::env::var("MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "2048".to_string())
                .parse()
                .unwrap_or(2048),
            e2ee_enabled: std::env::var("E2EE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}
