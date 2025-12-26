//! IVR Flow Definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::nodes::IvrNode;

/// IVR Flow Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrFlow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub entry_node: String,
    pub nodes: HashMap<String, IvrNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Active IVR Session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrSession {
    pub session_id: String,
    pub call_id: String,
    pub flow_id: String,
    pub current_node: String,
    pub variables: HashMap<String, serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub last_input: Option<String>,
    pub input_attempts: u32,
}

impl IvrFlow {
    /// Create a new IVR flow
    pub fn new(name: &str, entry_node: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: None,
            entry_node: entry_node.to_string(),
            nodes: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Add a node to the flow
    pub fn add_node(&mut self, node: IvrNode) {
        self.nodes.insert(node.id.clone(), node);
        self.updated_at = Utc::now();
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&IvrNode> {
        self.nodes.get(id)
    }

    /// Validate the flow (all branches lead to valid nodes)
    pub fn validate(&self) -> Result<(), FlowValidationError> {
        // Check entry node exists
        if !self.nodes.contains_key(&self.entry_node) {
            return Err(FlowValidationError::MissingEntryNode);
        }

        // Check all node references are valid
        for (id, node) in &self.nodes {
            for next_id in node.get_next_nodes() {
                if !self.nodes.contains_key(&next_id) {
                    return Err(FlowValidationError::InvalidNodeReference {
                        from: id.clone(),
                        to: next_id,
                    });
                }
            }
        }

        Ok(())
    }
}

impl IvrSession {
    /// Create a new session for a call
    pub fn new(call_id: &str, flow: &IvrFlow) -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            call_id: call_id.to_string(),
            flow_id: flow.id.clone(),
            current_node: flow.entry_node.clone(),
            variables: HashMap::new(),
            started_at: Utc::now(),
            last_input: None,
            input_attempts: 0,
        }
    }

    /// Set a variable
    pub fn set_variable(&mut self, key: &str, value: serde_json::Value) {
        self.variables.insert(key.to_string(), value);
    }

    /// Get a variable
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    /// Move to next node
    pub fn move_to(&mut self, node_id: &str) {
        self.current_node = node_id.to_string();
        self.input_attempts = 0;
    }

    /// Record input attempt
    pub fn record_input(&mut self, input: &str) {
        self.last_input = Some(input.to_string());
        self.input_attempts += 1;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FlowValidationError {
    #[error("Entry node does not exist")]
    MissingEntryNode,

    #[error("Invalid node reference from {from} to {to}")]
    InvalidNodeReference { from: String, to: String },
}
