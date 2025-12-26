//! RCS Configuration

use brivas_core::Result;

#[derive(Debug, Clone)]
pub struct RcsConfig {
    pub pop_id: String,
    pub http_bind: String,
    pub grpc_bind: String,
    pub lumadb_url: String,
    pub jibe_hub_url: String,
    pub jibe_api_key: Option<String>,
    pub samsung_rnc_url: Option<String>,
    pub mavenir_url: Option<String>,
    pub sms_fallback_enabled: bool,
    pub capability_cache_ttl_secs: u64,
}

impl RcsConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            pop_id: std::env::var("POP_ID").unwrap_or_else(|_| "local".to_string()),
            http_bind: std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            grpc_bind: std::env::var("GRPC_BIND").unwrap_or_else(|_| "0.0.0.0:9090".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            jibe_hub_url: std::env::var("JIBE_HUB_URL")
                .unwrap_or_else(|_| "https://rcsbusinessmessaging.googleapis.com".to_string()),
            jibe_api_key: std::env::var("JIBE_API_KEY").ok(),
            samsung_rnc_url: std::env::var("SAMSUNG_RNC_URL").ok(),
            mavenir_url: std::env::var("MAVENIR_URL").ok(),
            sms_fallback_enabled: std::env::var("SMS_FALLBACK_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            capability_cache_ttl_secs: std::env::var("CAPABILITY_CACHE_TTL_SECS")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .unwrap_or(86400),
        })
    }
}
