//! Workflow Registry
//!
//! Pre-configured Dify workflows for platform automation

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::{DifyClient, DifyError};

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Workflow not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Dify error: {0}")]
    Dify(#[from] DifyError),
}

pub type Result<T> = std::result::Result<T, WorkflowError>;

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dify_workflow_id: String,
    pub inputs: Vec<WorkflowInput>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    pub name: String,
    pub input_type: String,
    pub required: bool,
    pub description: String,
}

/// Workflow registry
pub struct WorkflowRegistry {
    client: Arc<DifyClient>,
    workflows: Vec<Workflow>,
}

impl WorkflowRegistry {
    pub fn new(client: Arc<DifyClient>) -> Self {
        // Pre-configured workflows
        let workflows = vec![
            Workflow {
                id: "campaign_builder".to_string(),
                name: "AI Campaign Builder".to_string(),
                description: "Create SMS/RCS campaigns from natural language descriptions".to_string(),
                dify_workflow_id: "campaign-builder-workflow".to_string(),
                inputs: vec![
                    WorkflowInput {
                        name: "description".to_string(),
                        input_type: "string".to_string(),
                        required: true,
                        description: "Natural language description of the campaign".to_string(),
                    },
                    WorkflowInput {
                        name: "target_audience".to_string(),
                        input_type: "string".to_string(),
                        required: false,
                        description: "Target audience segment".to_string(),
                    },
                    WorkflowInput {
                        name: "channel".to_string(),
                        input_type: "string".to_string(),
                        required: false,
                        description: "Channel: sms, rcs, whatsapp".to_string(),
                    },
                    WorkflowInput {
                        name: "budget".to_string(),
                        input_type: "number".to_string(),
                        required: false,
                        description: "Campaign budget".to_string(),
                    },
                ],
                outputs: vec!["campaign_config".to_string(), "message_template".to_string(), "schedule".to_string()],
            },
            Workflow {
                id: "billing_dispute".to_string(),
                name: "Billing Dispute Resolution".to_string(),
                description: "Automated billing dispute resolution with human-in-the-loop".to_string(),
                dify_workflow_id: "billing-dispute-workflow".to_string(),
                inputs: vec![
                    WorkflowInput {
                        name: "dispute_reason".to_string(),
                        input_type: "string".to_string(),
                        required: true,
                        description: "Customer's dispute reason".to_string(),
                    },
                    WorkflowInput {
                        name: "account_id".to_string(),
                        input_type: "string".to_string(),
                        required: true,
                        description: "Customer account ID".to_string(),
                    },
                    WorkflowInput {
                        name: "transaction_ids".to_string(),
                        input_type: "array".to_string(),
                        required: false,
                        description: "Disputed transaction IDs".to_string(),
                    },
                ],
                outputs: vec!["resolution".to_string(), "refund_amount".to_string(), "escalation_needed".to_string()],
            },
            Workflow {
                id: "incident_triage".to_string(),
                name: "Incident Triage".to_string(),
                description: "Automatically triage and categorize incidents".to_string(),
                dify_workflow_id: "incident-triage-workflow".to_string(),
                inputs: vec![
                    WorkflowInput {
                        name: "error_message".to_string(),
                        input_type: "string".to_string(),
                        required: true,
                        description: "Error message or symptom".to_string(),
                    },
                    WorkflowInput {
                        name: "service".to_string(),
                        input_type: "string".to_string(),
                        required: true,
                        description: "Affected service".to_string(),
                    },
                    WorkflowInput {
                        name: "metrics".to_string(),
                        input_type: "object".to_string(),
                        required: false,
                        description: "Related metrics data".to_string(),
                    },
                ],
                outputs: vec!["severity".to_string(), "category".to_string(), "recommended_playbook".to_string()],
            },
            Workflow {
                id: "sender_id_approval".to_string(),
                name: "Sender ID Approval".to_string(),
                description: "Validate and process sender ID registration requests".to_string(),
                dify_workflow_id: "sender-id-approval-workflow".to_string(),
                inputs: vec![
                    WorkflowInput {
                        name: "sender_id".to_string(),
                        input_type: "string".to_string(),
                        required: true,
                        description: "Requested sender ID".to_string(),
                    },
                    WorkflowInput {
                        name: "company_name".to_string(),
                        input_type: "string".to_string(),
                        required: true,
                        description: "Company name".to_string(),
                    },
                    WorkflowInput {
                        name: "use_case".to_string(),
                        input_type: "string".to_string(),
                        required: true,
                        description: "Intended use case".to_string(),
                    },
                ],
                outputs: vec!["approved".to_string(), "reason".to_string(), "compliance_notes".to_string()],
            },
        ];
        
        Self { client, workflows }
    }
    
    /// List all available workflows
    pub fn list(&self) -> Vec<Workflow> {
        self.workflows.clone()
    }
    
    /// Get workflow by ID
    pub fn get(&self, id: &str) -> Option<&Workflow> {
        self.workflows.iter().find(|w| w.id == id)
    }
    
    /// Run a workflow
    pub async fn run(
        &self,
        workflow_id: &str,
        inputs: serde_json::Value,
    ) -> Result<WorkflowResult> {
        let workflow = self.get(workflow_id)
            .ok_or_else(|| WorkflowError::NotFound(workflow_id.to_string()))?;
        
        // Validate required inputs
        for input in &workflow.inputs {
            if input.required && !inputs.get(&input.name).is_some() {
                return Err(WorkflowError::Validation(
                    format!("Missing required input: {}", input.name)
                ));
            }
        }
        
        // In production, call actual Dify workflow
        // For now, return simulated response
        let outputs = match workflow_id {
            "campaign_builder" => {
                let description = inputs.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("promotional campaign");
                
                serde_json::json!({
                    "campaign_config": {
                        "name": format!("AI-Generated: {}", description),
                        "channel": inputs.get("channel").unwrap_or(&serde_json::json!("sms")),
                        "target_segment": inputs.get("target_audience").unwrap_or(&serde_json::json!("all_users")),
                        "budget": inputs.get("budget").unwrap_or(&serde_json::json!(1000.0))
                    },
                    "message_template": "Generated message based on your description...",
                    "schedule": {
                        "start": chrono::Utc::now().to_rfc3339(),
                        "frequency": "once"
                    }
                })
            }
            "incident_triage" => {
                serde_json::json!({
                    "severity": "medium",
                    "category": "service_degradation",
                    "recommended_playbook": "service_restart",
                    "analysis": "Based on the error pattern, this appears to be a recoverable issue."
                })
            }
            _ => serde_json::json!({
                "status": "completed",
                "workflow_id": workflow_id
            }),
        };
        
        Ok(WorkflowResult {
            workflow_id: workflow_id.to_string(),
            run_id: uuid::Uuid::new_v4().to_string(),
            status: "completed".to_string(),
            outputs,
            elapsed_ms: 150,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub run_id: String,
    pub status: String,
    pub outputs: serde_json::Value,
    pub elapsed_ms: u64,
}
