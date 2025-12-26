//! STIR/SHAKEN Service Configuration

use brivas_core::Result;

#[derive(Debug, Clone)]
pub struct StirShakenConfig {
    /// gRPC bind address (primary API)
    pub grpc_bind: String,
    /// HTTP bind address (management API)
    pub http_bind: String,
    /// LumaDB connection URL
    pub lumadb_url: String,
    /// STI-CA URLs for certificate chain validation
    pub sti_ca_urls: Vec<String>,
    /// HSM configuration (optional)
    pub hsm_enabled: bool,
    pub hsm_slot: Option<u32>,
    /// TLS configuration for gRPC
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub client_ca_path: String,
    /// PASSporT validity window (seconds)
    pub passport_validity_secs: u64,
    /// Certificate cache TTL (seconds)
    pub cert_cache_ttl_secs: u64,
    /// CRL cache TTL (seconds)
    pub crl_cache_ttl_secs: u64,
    /// PoP identifier
    pub pop_id: String,
}

impl StirShakenConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            grpc_bind: std::env::var("STIR_SHAKEN_GRPC_BIND")
                .unwrap_or_else(|_| "0.0.0.0:50051".to_string()),
            http_bind: std::env::var("STIR_SHAKEN_HTTP_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            lumadb_url: std::env::var("LUMADB_URL")
                .unwrap_or_else(|_| "postgresql://brivas:brivas@lumadb:5432/brivas".to_string()),
            sti_ca_urls: std::env::var("STI_CA_URLS")
                .unwrap_or_else(|_| "https://authenticate.iconectiv.com".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            hsm_enabled: std::env::var("HSM_ENABLED")
                .map(|v| v == "true")
                .unwrap_or(false),
            hsm_slot: std::env::var("HSM_SLOT")
                .ok()
                .and_then(|s| s.parse().ok()),
            tls_cert_path: std::env::var("TLS_CERT_PATH")
                .unwrap_or_else(|_| "/etc/stir-shaken/tls/server.pem".to_string()),
            tls_key_path: std::env::var("TLS_KEY_PATH")
                .unwrap_or_else(|_| "/etc/stir-shaken/tls/server-key.pem".to_string()),
            client_ca_path: std::env::var("CLIENT_CA_PATH")
                .unwrap_or_else(|_| "/etc/stir-shaken/tls/ca.pem".to_string()),
            passport_validity_secs: std::env::var("PASSPORT_VALIDITY_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
            cert_cache_ttl_secs: std::env::var("CERT_CACHE_TTL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
            crl_cache_ttl_secs: std::env::var("CRL_CACHE_TTL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(21600),
            pop_id: std::env::var("POP_ID")
                .unwrap_or_else(|_| "default".to_string()),
        })
    }
}
