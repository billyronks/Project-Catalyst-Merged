//! MCP Gateway Configuration

use brivas_core::Result;

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub http_bind: String,
    pub lumadb_url: String,
    pub enable_stdio: bool,
    pub enable_sse: bool,
    pub api_key: Option<String>,
}

impl McpConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_bind: std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            enable_stdio: std::env::var("MCP_ENABLE_STDIO")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            enable_sse: std::env::var("MCP_ENABLE_SSE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            api_key: std::env::var("MCP_API_KEY").ok(),
        })
    }
}
