//! AIOps Engine Configuration

use brivas_core::Result;

#[derive(Debug, Clone)]
pub struct AiOpsConfig {
    pub http_bind: String,
    pub lumadb_url: String,
    pub check_interval_secs: u64,
    pub playbooks_dir: String,
    pub alert_webhook_url: Option<String>,
    pub pagerduty_key: Option<String>,
    pub slack_webhook_url: Option<String>,
}

impl AiOpsConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_bind: std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            check_interval_secs: std::env::var("CHECK_INTERVAL_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            playbooks_dir: std::env::var("PLAYBOOKS_DIR")
                .unwrap_or_else(|_| "/etc/aiops/playbooks".to_string()),
            alert_webhook_url: std::env::var("ALERT_WEBHOOK_URL").ok(),
            pagerduty_key: std::env::var("PAGERDUTY_KEY").ok(),
            slack_webhook_url: std::env::var("SLACK_WEBHOOK_URL").ok(),
        })
    }
}
