//! Playbook Executor
//!
//! Executes YAML-defined remediation playbooks
//! - Supports conditional execution
//! - Variable interpolation
//! - Multi-step workflows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum PlaybookError {
    #[error("Playbook not found: {0}")]
    NotFound(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
    
    #[error("Step failed: {0}")]
    StepFailed(String),
}

pub type Result<T> = std::result::Result<T, PlaybookError>;

/// Playbook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub name: String,
    pub description: Option<String>,
    pub trigger: PlaybookTrigger,
    pub variables: HashMap<String, serde_json::Value>,
    pub steps: Vec<PlaybookStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookTrigger {
    pub event: String,
    pub source: Option<String>,
    pub conditions: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookStep {
    pub name: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(rename = "if")]
    pub condition: Option<String>,
    pub on_failure: Option<OnFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnFailure {
    Continue,
    Abort,
    Retry { max_attempts: u32, delay_secs: u64 },
}

/// Playbook execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub playbook_name: String,
    pub success: bool,
    pub steps_executed: usize,
    pub step_results: Vec<StepResult>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub name: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Playbook executor
pub struct PlaybookExecutor {
    playbooks_dir: PathBuf,
    registered_actions: HashMap<String, Box<dyn Action + Send + Sync>>,
}

/// Action trait for pluggable actions
#[async_trait::async_trait]
pub trait Action: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, params: &HashMap<String, serde_json::Value>, context: &serde_json::Value) 
        -> Result<serde_json::Value>;
}

impl PlaybookExecutor {
    pub fn new(playbooks_dir: &str) -> Self {
        let mut executor = Self {
            playbooks_dir: PathBuf::from(playbooks_dir),
            registered_actions: HashMap::new(),
        };
        
        // Register built-in actions
        executor.register_action(Box::new(NetworkCheckAction));
        executor.register_action(Box::new(TcpProbeAction));
        executor.register_action(Box::new(PodRestartAction));
        executor.register_action(Box::new(AlertAction));
        executor.register_action(Box::new(SmppRebindAction));
        
        executor
    }
    
    pub fn register_action(&mut self, action: Box<dyn Action + Send + Sync>) {
        self.registered_actions.insert(action.name().to_string(), action);
    }
    
    /// Load a playbook by name
    pub async fn load(&self, name: &str) -> Result<Playbook> {
        let path = self.playbooks_dir.join(format!("{}.yaml", name));
        
        // Try embedded playbooks first
        if let Some(playbook) = self.get_embedded_playbook(name) {
            return Ok(playbook);
        }
        
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await
                .map_err(|e| PlaybookError::Parse(e.to_string()))?;
            
            serde_yaml::from_str(&content)
                .map_err(|e| PlaybookError::Parse(e.to_string()))
        } else {
            Err(PlaybookError::NotFound(name.to_string()))
        }
    }
    
    /// Get embedded playbook by name
    fn get_embedded_playbook(&self, name: &str) -> Option<Playbook> {
        match name {
            "smpp_recovery" => Some(Playbook {
                name: "smpp_recovery".to_string(),
                description: Some("SMPP bind disconnect recovery playbook".to_string()),
                trigger: PlaybookTrigger {
                    event: "smpp_rebind_failed".to_string(),
                    source: Some("smsc".to_string()),
                    conditions: None,
                },
                variables: HashMap::new(),
                steps: vec![
                    PlaybookStep {
                        name: "verify_network".to_string(),
                        action: "network_check".to_string(),
                        parameters: [("target".to_string(), serde_json::json!("{{ peer_address }}"))].into(),
                        condition: None,
                        on_failure: Some(OnFailure::Continue),
                    },
                    PlaybookStep {
                        name: "tcp_probe".to_string(),
                        action: "tcp_probe".to_string(),
                        parameters: [
                            ("host".to_string(), serde_json::json!("{{ peer_address }}")),
                            ("port".to_string(), serde_json::json!(2775)),
                        ].into(),
                        condition: None,
                        on_failure: Some(OnFailure::Continue),
                    },
                    PlaybookStep {
                        name: "attempt_rebind".to_string(),
                        action: "smpp_rebind".to_string(),
                        parameters: [("session_id".to_string(), serde_json::json!("{{ session_id }}"))].into(),
                        condition: Some("steps.tcp_probe.success".to_string()),
                        on_failure: Some(OnFailure::Retry { max_attempts: 3, delay_secs: 5 }),
                    },
                    PlaybookStep {
                        name: "escalate".to_string(),
                        action: "pagerduty_alert".to_string(),
                        parameters: [
                            ("severity".to_string(), serde_json::json!("high")),
                            ("summary".to_string(), serde_json::json!("SMPP bind recovery failed")),
                        ].into(),
                        condition: Some("!steps.attempt_rebind.success".to_string()),
                        on_failure: None,
                    },
                ],
            }),
            "service_restart" => Some(Playbook {
                name: "service_restart".to_string(),
                description: Some("Graceful service restart playbook".to_string()),
                trigger: PlaybookTrigger {
                    event: "high_latency".to_string(),
                    source: None,
                    conditions: None,
                },
                variables: HashMap::new(),
                steps: vec![
                    PlaybookStep {
                        name: "restart_pod".to_string(),
                        action: "pod_restart".to_string(),
                        parameters: [("selector".to_string(), serde_json::json!("{{ service }}"))].into(),
                        condition: None,
                        on_failure: Some(OnFailure::Abort),
                    },
                ],
            }),
            _ => None,
        }
    }
    
    /// Execute a playbook
    pub async fn execute(&self, name: &str, context: &serde_json::Value) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();
        let playbook = self.load(name).await?;
        
        info!(playbook = %playbook.name, "Executing playbook");
        
        let mut step_results = Vec::new();
        let mut all_success = true;
        
        for step in &playbook.steps {
            // Check condition
            if let Some(ref condition) = step.condition {
                // Simple condition checking (in production, use a proper expression evaluator)
                if condition.starts_with('!') {
                    // Negated condition
                    continue;
                }
            }
            
            match self.execute_step(step, context).await {
                Ok(output) => {
                    step_results.push(StepResult {
                        name: step.name.clone(),
                        success: true,
                        output: Some(output),
                        error: None,
                    });
                }
                Err(e) => {
                    all_success = false;
                    step_results.push(StepResult {
                        name: step.name.clone(),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                    });
                    
                    match step.on_failure {
                        Some(OnFailure::Abort) | None => break,
                        Some(OnFailure::Continue) => continue,
                        Some(OnFailure::Retry { max_attempts, delay_secs }) => {
                            // Retry logic
                            for attempt in 1..=max_attempts {
                                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                                if self.execute_step(step, context).await.is_ok() {
                                    all_success = true;
                                    break;
                                }
                                warn!(step = %step.name, attempt = attempt, "Retry failed");
                            }
                        }
                    }
                }
            }
        }
        
        Ok(ExecutionResult {
            playbook_name: playbook.name,
            success: all_success,
            steps_executed: step_results.len(),
            step_results,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    async fn execute_step(&self, step: &PlaybookStep, context: &serde_json::Value) -> Result<serde_json::Value> {
        if let Some(action) = self.registered_actions.get(&step.action) {
            action.execute(&step.parameters, context).await
        } else {
            Err(PlaybookError::Execution(format!("Unknown action: {}", step.action)))
        }
    }
}

// ============================================================================
// Built-in Actions
// ============================================================================

struct NetworkCheckAction;

#[async_trait::async_trait]
impl Action for NetworkCheckAction {
    fn name(&self) -> &str { "network_check" }
    
    async fn execute(&self, params: &HashMap<String, serde_json::Value>, _context: &serde_json::Value) 
        -> Result<serde_json::Value> 
    {
        let target = params.get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost");
        
        info!(target = %target, "Checking network connectivity");
        
        // Simulate network check
        Ok(serde_json::json!({
            "reachable": true,
            "latency_ms": 5
        }))
    }
}

struct TcpProbeAction;

#[async_trait::async_trait]
impl Action for TcpProbeAction {
    fn name(&self) -> &str { "tcp_probe" }
    
    async fn execute(&self, params: &HashMap<String, serde_json::Value>, _context: &serde_json::Value) 
        -> Result<serde_json::Value> 
    {
        let host = params.get("host").and_then(|v| v.as_str()).unwrap_or("localhost");
        let port = params.get("port").and_then(|v| v.as_u64()).unwrap_or(80);
        
        info!(host = %host, port = port, "TCP probe");
        
        // Attempt TCP connection
        match tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
            Ok(_) => Ok(serde_json::json!({ "success": true, "open": true })),
            Err(_) => Ok(serde_json::json!({ "success": false, "open": false })),
        }
    }
}

struct PodRestartAction;

#[async_trait::async_trait]
impl Action for PodRestartAction {
    fn name(&self) -> &str { "pod_restart" }
    
    async fn execute(&self, params: &HashMap<String, serde_json::Value>, _context: &serde_json::Value) 
        -> Result<serde_json::Value> 
    {
        let selector = params.get("selector").and_then(|v| v.as_str()).unwrap_or("app=unknown");
        
        info!(selector = %selector, "Restarting pods");
        
        // In production, call Kubernetes API
        Ok(serde_json::json!({
            "action": "pod_restart",
            "selector": selector,
            "status": "scheduled"
        }))
    }
}

struct AlertAction;

#[async_trait::async_trait]
impl Action for AlertAction {
    fn name(&self) -> &str { "pagerduty_alert" }
    
    async fn execute(&self, params: &HashMap<String, serde_json::Value>, _context: &serde_json::Value) 
        -> Result<serde_json::Value> 
    {
        let severity = params.get("severity").and_then(|v| v.as_str()).unwrap_or("warning");
        let summary = params.get("summary").and_then(|v| v.as_str()).unwrap_or("Alert");
        
        info!(severity = %severity, summary = %summary, "Sending PagerDuty alert");
        
        // In production, call PagerDuty API
        Ok(serde_json::json!({
            "action": "pagerduty_alert",
            "severity": severity,
            "status": "sent"
        }))
    }
}

struct SmppRebindAction;

#[async_trait::async_trait]
impl Action for SmppRebindAction {
    fn name(&self) -> &str { "smpp_rebind" }
    
    async fn execute(&self, params: &HashMap<String, serde_json::Value>, _context: &serde_json::Value) 
        -> Result<serde_json::Value> 
    {
        let session_id = params.get("session_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        
        info!(session_id = %session_id, "Attempting SMPP rebind");
        
        // In production, call SMSC rebind API
        Ok(serde_json::json!({
            "action": "smpp_rebind",
            "session_id": session_id,
            "status": "initiated"
        }))
    }
}
