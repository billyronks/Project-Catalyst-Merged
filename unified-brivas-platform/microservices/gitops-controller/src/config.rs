//! GitOps Controller Configuration

use brivas_core::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GitOpsConfig {
    pub http_bind: String,
    pub lumadb_url: String,
    pub sync_interval_secs: u64,
    pub repos_dir: String,
    pub repositories: Vec<RepositoryConfig>,
    pub aiops_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub url: String,
    pub branch: String,
    pub path: Option<String>,
    pub ssh_key_path: Option<String>,
}

impl GitOpsConfig {
    pub fn from_env() -> Result<Self> {
        // Parse GITOPS_REPOSITORIES from JSON or comma-separated URLs
        let repos_str = std::env::var("GITOPS_REPOSITORIES").unwrap_or_default();
        let repositories = if repos_str.starts_with('[') {
            serde_json::from_str(&repos_str).unwrap_or_default()
        } else {
            repos_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|url| RepositoryConfig {
                    url: url.trim().to_string(),
                    branch: "main".to_string(),
                    path: None,
                    ssh_key_path: None,
                })
                .collect()
        };
        
        Ok(Self {
            http_bind: std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            lumadb_url: std::env::var("LUMADB_URL").unwrap_or_else(|_| {
                "postgres://brivas:password@localhost:5432/brivas".to_string()
            }),
            sync_interval_secs: std::env::var("SYNC_INTERVAL_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            repos_dir: std::env::var("GITOPS_REPOS_DIR")
                .unwrap_or_else(|_| "/var/lib/gitops/repos".to_string()),
            repositories,
            aiops_endpoint: std::env::var("AIOPS_ENDPOINT").ok(),
        })
    }
}
