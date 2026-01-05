//! Application Manifest Types
//!
//! ArgoCD-compatible application manifest definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ArgoCD-style Application manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationManifest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: AppMetadata,
    pub spec: AppSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub name: String,
    pub namespace: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSpec {
    pub project: Option<String>,
    pub source: AppSource,
    pub destination: AppDestination,
    #[serde(rename = "syncPolicy", default)]
    pub sync_policy: Option<SyncPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSource {
    #[serde(rename = "repoURL")]
    pub repo_url: String,
    #[serde(rename = "targetRevision")]
    pub target_revision: Option<String>,
    pub path: Option<String>,
    pub chart: Option<String>,
    pub helm: Option<HelmSource>,
    pub kustomize: Option<KustomizeSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmSource {
    #[serde(rename = "valueFiles", default)]
    pub value_files: Vec<String>,
    #[serde(default)]
    pub values: Option<String>,
    #[serde(default)]
    pub parameters: Vec<HelmParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmParameter {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KustomizeSource {
    #[serde(rename = "namePrefix")]
    pub name_prefix: Option<String>,
    #[serde(rename = "nameSuffix")]
    pub name_suffix: Option<String>,
    #[serde(default)]
    pub images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDestination {
    pub server: Option<String>,
    pub namespace: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPolicy {
    #[serde(default)]
    pub automated: Option<AutomatedSync>,
    #[serde(rename = "syncOptions", default)]
    pub sync_options: Vec<String>,
    #[serde(default)]
    pub retry: Option<RetryPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedSync {
    #[serde(default)]
    pub prune: bool,
    #[serde(rename = "selfHeal", default)]
    pub self_heal: bool,
    #[serde(rename = "allowEmpty", default)]
    pub allow_empty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub limit: i32,
    pub backoff: Option<BackoffPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackoffPolicy {
    pub duration: String,
    #[serde(rename = "maxDuration")]
    pub max_duration: Option<String>,
    pub factor: Option<i32>,
}

/// Brivas-specific configuration manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrivasConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: AppMetadata,
    pub spec: BrivasConfigSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrivasConfigSpec {
    pub service: String,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub secrets: HashMap<String, String>,
    #[serde(default)]
    pub replicas: Option<i32>,
    #[serde(default)]
    pub resources: Option<ResourceSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub limits: Option<ResourceLimits>,
    pub requests: Option<ResourceLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: Option<String>,
    pub memory: Option<String>,
}

impl ApplicationManifest {
    /// Generate a unique identifier for this application
    pub fn id(&self) -> String {
        format!(
            "{}/{}",
            self.metadata.namespace.as_deref().unwrap_or("default"),
            self.metadata.name
        )
    }
    
    /// Calculate content hash for drift detection
    pub fn content_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let content = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    /// Check if auto-sync is enabled
    pub fn auto_sync_enabled(&self) -> bool {
        self.spec.sync_policy
            .as_ref()
            .and_then(|p| p.automated.as_ref())
            .is_some()
    }
    
    /// Check if self-heal is enabled
    pub fn self_heal_enabled(&self) -> bool {
        self.spec.sync_policy
            .as_ref()
            .and_then(|p| p.automated.as_ref())
            .map(|a| a.self_heal)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_argocd_manifest() {
        let yaml = r#"
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: test-app
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/org/repo.git
    targetRevision: HEAD
    path: deploy
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
"#;
        
        let manifest: ApplicationManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.metadata.name, "test-app");
        assert_eq!(manifest.kind, "Application");
        assert!(manifest.auto_sync_enabled());
        assert!(manifest.self_heal_enabled());
    }
}
