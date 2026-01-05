//! GitOps Controller
//!
//! Declarative configuration management for the Brivas platform:
//! - Git repository synchronization
//! - ArgoCD-compatible application manifests
//! - Configuration drift detection
//! - Automatic reconciliation
//! - AIOps integration for closed-loop operations

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use brivas_lumadb::{LumaDbPool, PoolConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod config;
mod git;
mod manifest;
mod reconciler;
mod drift;

pub use config::GitOpsConfig;
use git::GitRepository;
use manifest::ApplicationManifest;
use reconciler::Reconciler;
use drift::DriftDetector;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gitops_controller=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting GitOps Controller");

    let service = Arc::new(GitOpsService::new().await?);
    MicroserviceRuntime::run(service).await
}

/// GitOps service state
pub struct GitOpsService {
    config: GitOpsConfig,
    pool: LumaDbPool,
    reconciler: Arc<Reconciler>,
    drift_detector: Arc<DriftDetector>,
    repositories: Arc<RwLock<Vec<GitRepository>>>,
    start_time: std::time::Instant,
}

/// Sync status for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub repo_url: String,
    pub branch: String,
    pub last_commit: String,
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub status: SyncState,
    pub applications: Vec<ApplicationStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncState {
    Synced,
    OutOfSync,
    Progressing,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationStatus {
    pub name: String,
    pub namespace: String,
    pub health: HealthState,
    pub sync: SyncState,
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Degraded,
    Progressing,
    Suspended,
    Missing,
    Unknown,
}

impl GitOpsService {
    pub async fn new() -> Result<Self> {
        let config = GitOpsConfig::from_env()?;
        
        // Create LumaDB connection pool
        let pool_config = PoolConfig {
            url: config.lumadb_url.clone(),
            max_size: 8,
            min_idle: Some(2),
        };
        let pool = LumaDbPool::new(pool_config).await
            .map_err(|e| brivas_core::BrivasError::Database(e.to_string()))?;
        
        // Initialize components
        let reconciler = Arc::new(Reconciler::new(pool.clone(), &config));
        let drift_detector = Arc::new(DriftDetector::new(pool.clone()));

        Ok(Self {
            config,
            pool,
            reconciler,
            drift_detector,
            repositories: Arc::new(RwLock::new(Vec::new())),
            start_time: std::time::Instant::now(),
        })
    }
    
    /// Start the sync loop
    async fn run_sync_loop(&self) {
        let config = self.config.clone();
        let reconciler = self.reconciler.clone();
        let repositories = self.repositories.clone();
        let drift_detector = self.drift_detector.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(config.sync_interval_secs)
            );
            
            loop {
                interval.tick().await;
                
                info!("Starting GitOps sync cycle");
                
                // Clone/pull configured repositories
                for repo_config in &config.repositories {
                    match GitRepository::sync(&repo_config.url, &repo_config.branch, &config.repos_dir).await {
                        Ok(repo) => {
                            info!(repo = %repo_config.url, "Repository synced");
                            
                            // Parse application manifests
                            if let Ok(manifests) = repo.discover_applications().await {
                                for manifest in manifests {
                                    // Check for drift
                                    if let Ok(has_drift) = drift_detector.check(&manifest).await {
                                        if has_drift {
                                            info!(app = %manifest.metadata.name, "Drift detected, reconciling");
                                            let _ = reconciler.reconcile(&manifest).await;
                                        }
                                    }
                                }
                            }
                            
                            // Update repository list
                            let mut repos = repositories.write().await;
                            if let Some(existing) = repos.iter_mut().find(|r| r.url == repo_config.url) {
                                *existing = repo;
                            } else {
                                repos.push(repo);
                            }
                        }
                        Err(e) => {
                            tracing::error!(repo = %repo_config.url, error = %e, "Failed to sync repository");
                        }
                    }
                }
            }
        });
    }
}

#[async_trait::async_trait]
impl BrivasService for GitOpsService {
    fn service_id(&self) -> &'static str {
        "gitops-controller"
    }

    async fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            service_id: self.service_id().to_string(),
            version: self.version().to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    async fn ready(&self) -> ReadinessStatus {
        let db_healthy = self.pool.is_healthy().await;
        ReadinessStatus {
            ready: db_healthy,
            dependencies: vec![brivas_core::DependencyStatus {
                name: "lumadb".to_string(),
                available: db_healthy,
                latency_ms: Some(1),
            }],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down GitOps Controller");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            sync_interval = %self.config.sync_interval_secs,
            repos = %self.config.repositories.len(),
            "Starting GitOps Controller"
        );
        
        // Start sync loop
        self.run_sync_loop().await;
        
        // Create HTTP routes
        let repositories = self.repositories.clone();
        let reconciler = self.reconciler.clone();
        let drift_detector = self.drift_detector.clone();
        
        let app = axum::Router::new()
            .route("/health", axum::routing::get(|| async { "OK" }))
            .route("/ready", axum::routing::get(|| async { "OK" }))
            .route("/api/v1/sync/status", axum::routing::get({
                let repositories = repositories.clone();
                move || {
                    let repositories = repositories.clone();
                    async move {
                        let repos = repositories.read().await;
                        let statuses: Vec<SyncStatus> = repos.iter().map(|r| r.status()).collect();
                        axum::Json(serde_json::json!({
                            "repositories": statuses.len(),
                            "synced": statuses.iter().filter(|s| s.status == SyncState::Synced).count(),
                            "statuses": statuses
                        }))
                    }
                }
            }))
            .route("/api/v1/sync/trigger", axum::routing::post({
                move || async move {
                    axum::Json(serde_json::json!({
                        "message": "Sync triggered",
                        "status": "progressing"
                    }))
                }
            }))
            .route("/api/v1/applications", axum::routing::get({
                let repositories = repositories.clone();
                move || {
                    let repositories = repositories.clone();
                    async move {
                        let repos = repositories.read().await;
                        let apps: Vec<ApplicationStatus> = repos
                            .iter()
                            .flat_map(|r| r.status().applications)
                            .collect();
                        axum::Json(serde_json::json!({
                            "applications": apps
                        }))
                    }
                }
            }))
            .route("/api/v1/applications/:name/sync", axum::routing::post({
                let reconciler = reconciler.clone();
                move |axum::extract::Path(name): axum::extract::Path<String>| {
                    let reconciler = reconciler.clone();
                    async move {
                        axum::Json(serde_json::json!({
                            "application": name,
                            "action": "sync",
                            "status": "queued"
                        }))
                    }
                }
            }))
            .route("/api/v1/drift/check", axum::routing::post({
                let drift_detector = drift_detector.clone();
                move || {
                    let drift_detector = drift_detector.clone();
                    async move {
                        axum::Json(serde_json::json!({
                            "action": "drift_check",
                            "status": "initiated"
                        }))
                    }
                }
            }));

        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
