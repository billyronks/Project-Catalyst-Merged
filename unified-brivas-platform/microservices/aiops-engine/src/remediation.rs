//! Remediation Orchestrator
//!
//! Coordinates playbook execution and tracks remediation state

use brivas_lumadb::LumaDbPool;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use crate::playbook::{ExecutionResult, PlaybookExecutor};

#[derive(Debug, Error)]
pub enum RemediationError {
    #[error("Playbook error: {0}")]
    Playbook(#[from] crate::playbook::PlaybookError),
    
    #[error("Database error: {0}")]
    Database(#[from] brivas_lumadb::LumaDbError),
    
    #[error("Remediation failed: {0}")]
    Failed(String),
}

pub type Result<T> = std::result::Result<T, RemediationError>;

/// Remediation orchestrator
pub struct RemediationOrchestrator {
    executor: Arc<PlaybookExecutor>,
    pool: LumaDbPool,
}

impl RemediationOrchestrator {
    pub fn new(executor: Arc<PlaybookExecutor>, pool: LumaDbPool) -> Self {
        Self { executor, pool }
    }
    
    /// Execute a remediation playbook
    pub async fn execute(&self, playbook_id: &str, context: &serde_json::Value) -> Result<ExecutionResult> {
        info!(playbook = %playbook_id, "Starting remediation");
        
        let result = self.executor.execute(playbook_id, context).await?;
        
        // Log execution result to database
        self.log_execution(&result).await?;
        
        if result.success {
            info!(playbook = %playbook_id, duration_ms = result.duration_ms, "Remediation successful");
        } else {
            info!(playbook = %playbook_id, "Remediation failed, escalating");
        }
        
        Ok(result)
    }
    
    async fn log_execution(&self, result: &ExecutionResult) -> Result<()> {
        // In production, log to remediation_history table
        Ok(())
    }
}
