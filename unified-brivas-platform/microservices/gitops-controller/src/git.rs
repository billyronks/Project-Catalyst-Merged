//! Git Repository Operations
//!
//! Handles cloning, pulling, and reading from Git repositories

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info};

use crate::manifest::ApplicationManifest;
use crate::{ApplicationStatus, HealthState, SyncState, SyncStatus};

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Clone failed: {0}")]
    CloneFailed(String),
    
    #[error("Pull failed: {0}")]
    PullFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Git2 error: {0}")]
    Git2(#[from] git2::Error),
    
    #[error("Manifest parse error: {0}")]
    ManifestParse(String),
}

pub type Result<T> = std::result::Result<T, GitError>;

/// Git repository wrapper
#[derive(Debug, Clone)]
pub struct GitRepository {
    pub url: String,
    pub branch: String,
    pub local_path: PathBuf,
    pub last_commit: String,
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub applications: Vec<ApplicationManifest>,
}

impl GitRepository {
    /// Sync a repository (clone or pull)
    pub async fn sync(url: &str, branch: &str, repos_dir: &str) -> Result<Self> {
        let repo_name = Self::url_to_name(url);
        let local_path = PathBuf::from(repos_dir).join(&repo_name);
        
        // Run git operations in blocking task
        let url_owned = url.to_string();
        let branch_owned = branch.to_string();
        let local_path_clone = local_path.clone();
        
        let last_commit = tokio::task::spawn_blocking(move || {
            if local_path_clone.exists() {
                Self::pull(&local_path_clone, &branch_owned)
            } else {
                Self::clone(&url_owned, &branch_owned, &local_path_clone)
            }
        })
        .await
        .map_err(|e| GitError::CloneFailed(e.to_string()))??;
        
        let mut repo = Self {
            url: url.to_string(),
            branch: branch.to_string(),
            local_path,
            last_commit,
            last_sync: chrono::Utc::now(),
            applications: Vec::new(),
        };
        
        // Discover applications
        repo.applications = repo.discover_applications().await.unwrap_or_default();
        
        Ok(repo)
    }
    
    /// Clone a repository
    fn clone(url: &str, branch: &str, path: &Path) -> Result<String> {
        info!(url = %url, path = ?path, "Cloning repository");
        
        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let repo = git2::Repository::clone(url, path)?;
        
        // Checkout specific branch if not default
        if branch != "main" && branch != "master" {
            let (object, reference) = repo.revparse_ext(&format!("origin/{}", branch))?;
            repo.checkout_tree(&object, None)?;
            if let Some(gref) = reference {
                repo.set_head(gref.name().unwrap_or("refs/heads/main"))?;
            }
        }
        
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        
        Ok(commit.id().to_string())
    }
    
    /// Pull latest changes
    fn pull(path: &Path, branch: &str) -> Result<String> {
        debug!(path = ?path, "Pulling repository");
        
        let repo = git2::Repository::open(path)?;
        
        // Fetch
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[branch], None, None)?;
        
        // Get fetch head
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let commit = fetch_head.peel_to_commit()?;
        
        // Fast-forward merge
        let refname = format!("refs/heads/{}", branch);
        if let Ok(mut reference) = repo.find_reference(&refname) {
            reference.set_target(commit.id(), "GitOps sync")?;
        }
        
        // Checkout
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        
        Ok(commit.id().to_string())
    }
    
    /// Discover ArgoCD-style application manifests
    pub async fn discover_applications(&self) -> Result<Vec<ApplicationManifest>> {
        let path = self.local_path.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut applications = Vec::new();
            
            // Look for Application manifests in standard locations
            let patterns = [
                "*.yaml",
                "*.yml",
                "apps/*.yaml",
                "apps/*.yml",
                "applications/*.yaml",
                "applications/*.yml",
                "manifests/*.yaml",
                "manifests/*.yml",
            ];
            
            for pattern in patterns {
                let full_pattern = path.join(pattern);
                if let Ok(entries) = glob::glob(full_pattern.to_str().unwrap_or("")) {
                    for entry in entries.flatten() {
                        if let Ok(content) = std::fs::read_to_string(&entry) {
                            // Try to parse as YAML
                            if let Ok(manifest) = serde_yaml::from_str::<ApplicationManifest>(&content) {
                                if manifest.kind == "Application" || manifest.kind == "BrivasApp" {
                                    debug!(path = ?entry, "Found application manifest");
                                    applications.push(manifest);
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(applications)
        })
        .await
        .map_err(|e| GitError::ManifestParse(e.to_string()))?
    }
    
    /// Get sync status
    pub fn status(&self) -> SyncStatus {
        SyncStatus {
            repo_url: self.url.clone(),
            branch: self.branch.clone(),
            last_commit: self.last_commit.clone(),
            last_sync: self.last_sync,
            status: SyncState::Synced,
            applications: self.applications.iter().map(|app| {
                ApplicationStatus {
                    name: app.metadata.name.clone(),
                    namespace: app.metadata.namespace.clone().unwrap_or_else(|| "default".to_string()),
                    health: HealthState::Healthy,
                    sync: SyncState::Synced,
                    revision: self.last_commit.clone(),
                }
            }).collect(),
        }
    }
    
    /// Convert URL to safe directory name
    fn url_to_name(url: &str) -> String {
        url.trim_start_matches("https://")
            .trim_start_matches("git@")
            .replace(['/', ':', '.'], "_")
            .trim_end_matches("_git")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_to_name() {
        assert_eq!(
            GitRepository::url_to_name("https://github.com/org/repo.git"),
            "github_com_org_repo"
        );
        assert_eq!(
            GitRepository::url_to_name("git@github.com:org/repo.git"),
            "github_com_org_repo"
        );
    }
}
