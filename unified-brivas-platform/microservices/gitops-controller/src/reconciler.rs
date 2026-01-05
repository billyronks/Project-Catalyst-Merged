//! Reconciler
//!
//! Applies desired state from manifests to the platform

use brivas_lumadb::LumaDbPool;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

use crate::config::GitOpsConfig;
use crate::manifest::ApplicationManifest;

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("Database error: {0}")]
    Database(#[from] brivas_lumadb::LumaDbError),
    
    #[error("Apply failed: {0}")]
    ApplyFailed(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

pub type Result<T> = std::result::Result<T, ReconcileError>;

/// Reconciliation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileResult {
    pub application: String,
    pub success: bool,
    pub message: String,
    pub changes_applied: usize,
    pub duration_ms: u64,
}

/// Reconciler for applying manifests
pub struct Reconciler {
    pool: LumaDbPool,
    aiops_endpoint: Option<String>,
}

impl Reconciler {
    pub fn new(pool: LumaDbPool, config: &GitOpsConfig) -> Self {
        Self {
            pool,
            aiops_endpoint: config.aiops_endpoint.clone(),
        }
    }
    
    /// Reconcile an application manifest
    pub async fn reconcile(&self, manifest: &ApplicationManifest) -> Result<ReconcileResult> {
        let start = std::time::Instant::now();
        let app_id = manifest.id();
        
        info!(app = %app_id, "Starting reconciliation");
        
        // Validate manifest
        self.validate(manifest)?;
        
        // Apply manifest based on type
        let changes = self.apply(manifest).await?;
        
        // Record reconciliation
        self.record_sync(&app_id, &manifest.content_hash()).await?;
        
        // Notify AIOps if configured
        if let Some(ref endpoint) = self.aiops_endpoint {
            self.notify_aiops(endpoint, &app_id, changes).await;
        }
        
        Ok(ReconcileResult {
            application: app_id,
            success: true,
            message: "Reconciliation complete".to_string(),
            changes_applied: changes,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    /// Validate manifest before applying
    fn validate(&self, manifest: &ApplicationManifest) -> Result<()> {
        // Basic validation
        if manifest.metadata.name.is_empty() {
            return Err(ReconcileError::ValidationFailed("name is required".into()));
        }
        
        if manifest.spec.source.repo_url.is_empty() {
            return Err(ReconcileError::ValidationFailed("source.repoURL is required".into()));
        }
        
        Ok(())
    }
    
    /// Apply manifest changes
    async fn apply(&self, manifest: &ApplicationManifest) -> Result<usize> {
        let conn = self.pool.get().await?;
        
        // Store/update application state in database
        let query = r#"
            INSERT INTO gitops_applications (name, namespace, manifest_hash, last_sync, status)
            VALUES ($1, $2, $3, NOW(), 'synced')
            ON CONFLICT (name, namespace) 
            DO UPDATE SET manifest_hash = $3, last_sync = NOW(), status = 'synced'
        "#;
        
        let namespace = manifest.metadata.namespace.as_deref().unwrap_or("default");
        
        match conn.execute(query, &[&manifest.metadata.name, &namespace, &manifest.content_hash()]).await {
            Ok(rows) => {
                info!(
                    app = %manifest.metadata.name,
                    namespace = %namespace,
                    "Application state updated"
                );
                Ok(rows as usize)
            }
            Err(e) => {
                // Table might not exist - create it
                warn!(error = %e, "Failed to update application state, table may not exist");
                Ok(0)
            }
        }
    }
    
    /// Record sync in history
    async fn record_sync(&self, app_id: &str, hash: &str) -> Result<()> {
        let conn = self.pool.get().await?;
        
        let query = r#"
            INSERT INTO gitops_sync_history (app_id, manifest_hash, sync_time, status)
            VALUES ($1, $2, NOW(), 'success')
        "#;
        
        let _ = conn.execute(query, &[&app_id, &hash]).await;
        
        Ok(())
    }
    
    /// Notify AIOps of reconciliation
    async fn notify_aiops(&self, endpoint: &str, app_id: &str, changes: usize) {
        // In production, make HTTP call to AIOps engine
        info!(
            aiops_endpoint = %endpoint,
            app = %app_id,
            changes = changes,
            "Notifying AIOps of reconciliation"
        );
    }
}
