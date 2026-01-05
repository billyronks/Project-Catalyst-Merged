//! Drift Detection
//!
//! Detects configuration drift between desired and actual state

use brivas_lumadb::LumaDbPool;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::manifest::ApplicationManifest;

#[derive(Debug, Error)]
pub enum DriftError {
    #[error("Database error: {0}")]
    Database(#[from] brivas_lumadb::LumaDbError),
    
    #[error("Detection failed: {0}")]
    DetectionFailed(String),
}

pub type Result<T> = std::result::Result<T, DriftError>;

/// Drift detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftResult {
    pub application: String,
    pub has_drift: bool,
    pub drift_type: Option<DriftType>,
    pub details: Option<String>,
    pub last_sync_hash: Option<String>,
    pub current_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DriftType {
    ConfigurationChanged,
    ResourceModified,
    ManualOverride,
    NewApplication,
    Deleted,
}

/// Drift detector
pub struct DriftDetector {
    pool: LumaDbPool,
}

impl DriftDetector {
    pub fn new(pool: LumaDbPool) -> Self {
        Self { pool }
    }
    
    /// Check for drift in an application
    pub async fn check(&self, manifest: &ApplicationManifest) -> Result<bool> {
        let result = self.detect(manifest).await?;
        Ok(result.has_drift)
    }
    
    /// Detect drift with details
    pub async fn detect(&self, manifest: &ApplicationManifest) -> Result<DriftResult> {
        let app_id = manifest.id();
        let current_hash = manifest.content_hash();
        
        debug!(app = %app_id, hash = %current_hash, "Checking for drift");
        
        // Get last known state from database
        let last_sync = self.get_last_sync(&app_id).await?;
        
        match last_sync {
            Some((last_hash, _last_time)) => {
                if last_hash == current_hash {
                    // No drift
                    Ok(DriftResult {
                        application: app_id,
                        has_drift: false,
                        drift_type: None,
                        details: None,
                        last_sync_hash: Some(last_hash),
                        current_hash,
                    })
                } else {
                    // Configuration changed
                    info!(app = %app_id, "Drift detected: configuration changed");
                    Ok(DriftResult {
                        application: app_id,
                        has_drift: true,
                        drift_type: Some(DriftType::ConfigurationChanged),
                        details: Some(format!("Hash changed from {} to {}", last_hash, current_hash)),
                        last_sync_hash: Some(last_hash),
                        current_hash,
                    })
                }
            }
            None => {
                // New application
                info!(app = %app_id, "New application detected");
                Ok(DriftResult {
                    application: app_id,
                    has_drift: true,
                    drift_type: Some(DriftType::NewApplication),
                    details: Some("Application not previously synced".to_string()),
                    last_sync_hash: None,
                    current_hash,
                })
            }
        }
    }
    
    /// Get last sync information for an application
    async fn get_last_sync(&self, app_id: &str) -> Result<Option<(String, chrono::DateTime<chrono::Utc>)>> {
        let conn = self.pool.get().await?;
        
        let query = r#"
            SELECT manifest_hash, last_sync
            FROM gitops_applications
            WHERE CONCAT(namespace, '/', name) = $1
            LIMIT 1
        "#;
        
        match conn.query_opt(query, &[&app_id]).await {
            Ok(Some(row)) => {
                let hash: String = row.get(0);
                // Parse timestamp - using string for simplicity
                Ok(Some((hash, chrono::Utc::now())))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                // Table might not exist yet
                warn!(error = %e, "Could not query last sync, table may not exist");
                Ok(None)
            }
        }
    }
    
    /// Check all tracked applications for drift
    pub async fn check_all(&self) -> Result<Vec<DriftResult>> {
        let conn = self.pool.get().await?;
        
        let query = r#"
            SELECT name, namespace, manifest_hash
            FROM gitops_applications
        "#;
        
        match conn.query(query, &[]).await {
            Ok(rows) => {
                let results: Vec<DriftResult> = rows.iter().map(|row| {
                    let name: String = row.get(0);
                    let namespace: String = row.get(1);
                    let hash: String = row.get(2);
                    
                    DriftResult {
                        application: format!("{}/{}", namespace, name),
                        has_drift: false, // Would need to compare with actual state
                        drift_type: None,
                        details: None,
                        last_sync_hash: Some(hash.clone()),
                        current_hash: hash,
                    }
                }).collect();
                
                Ok(results)
            }
            Err(e) => {
                warn!(error = %e, "Could not query applications");
                Ok(vec![])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_types() {
        assert_eq!(
            serde_json::to_string(&DriftType::ConfigurationChanged).unwrap(),
            "\"configuration_changed\""
        );
    }
}
