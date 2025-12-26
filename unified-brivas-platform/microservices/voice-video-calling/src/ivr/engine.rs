//! IVR Engine
//!
//! Executes IVR flows by controlling FreeSWITCH media sessions.

use std::sync::Arc;
use dashmap::DashMap;

use crate::VoiceIvrConfig;
use super::flow::{IvrFlow, IvrSession};
use super::nodes::{IvrNode, IvrNodeType};

/// IVR Execution Result
#[derive(Debug, Clone)]
pub enum IvrResult {
    Completed,
    Transferred,
    HungUp,
    Error(String),
}

/// IVR Engine
pub struct IvrEngine {
    #[allow(dead_code)]
    config: VoiceIvrConfig,
    flows: Arc<DashMap<String, IvrFlow>>,
    sessions: Arc<DashMap<String, IvrSession>>,
}

impl IvrEngine {
    pub async fn new(config: &VoiceIvrConfig) -> brivas_core::Result<Self> {
        Ok(Self {
            config: config.clone(),
            flows: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        })
    }

    /// Create a new IVR flow
    pub fn create_flow(&self, flow: IvrFlow) -> Result<String, IvrError> {
        flow.validate().map_err(|e| IvrError::InvalidFlow(e.to_string()))?;
        let id = flow.id.clone();
        self.flows.insert(id.clone(), flow);
        Ok(id)
    }

    /// Get a flow by ID
    pub fn get_flow(&self, id: &str) -> Option<IvrFlow> {
        self.flows.get(id).map(|f| f.value().clone())
    }

    /// List all flows
    pub fn list_flows(&self) -> Vec<IvrFlow> {
        self.flows.iter().map(|f| f.value().clone()).collect()
    }

    /// Delete a flow
    pub fn delete_flow(&self, id: &str) -> bool {
        self.flows.remove(id).is_some()
    }

    /// Start IVR session for a call
    pub async fn start_session(
        &self,
        call_id: &str,
        flow_id: &str,
    ) -> Result<String, IvrError> {
        let flow = self.flows
            .get(flow_id)
            .ok_or(IvrError::FlowNotFound)?
            .value()
            .clone();

        let session = IvrSession::new(call_id, &flow);
        let session_id = session.session_id.clone();
        
        self.sessions.insert(session_id.clone(), session);

        // Start execution in background
        let sessions = self.sessions.clone();
        let flows = self.flows.clone();
        let sid = session_id.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::execute_session(&sessions, &flows, &sid).await {
                tracing::error!(session_id = %sid, error = %e, "IVR session failed");
            }
        });

        Ok(session_id)
    }

    /// Execute IVR session
    async fn execute_session(
        sessions: &DashMap<String, IvrSession>,
        flows: &DashMap<String, IvrFlow>,
        session_id: &str,
    ) -> Result<IvrResult, IvrError> {
        loop {
            let session = sessions
                .get(session_id)
                .ok_or(IvrError::SessionNotFound)?
                .value()
                .clone();

            let flow = flows
                .get(&session.flow_id)
                .ok_or(IvrError::FlowNotFound)?
                .value()
                .clone();

            let node = flow
                .get_node(&session.current_node)
                .ok_or(IvrError::NodeNotFound)?
                .clone();

            match Self::execute_node(&node, &session).await? {
                NodeResult::Next(next_node) => {
                    if let Some(mut s) = sessions.get_mut(session_id) {
                        s.move_to(&next_node);
                    }
                }
                NodeResult::Terminal(result) => {
                    sessions.remove(session_id);
                    return Ok(result);
                }
                NodeResult::WaitForInput => {
                    // Input will be processed by external event
                    return Ok(IvrResult::Completed);
                }
            }
        }
    }

    /// Execute a single node
    async fn execute_node(node: &IvrNode, _session: &IvrSession) -> Result<NodeResult, IvrError> {
        match &node.node_type {
            IvrNodeType::PlayAudio { audio_url, next } => {
                // TODO: Call FreeSWITCH to play audio
                tracing::info!(audio_url = %audio_url, "Playing audio");
                Ok(NodeResult::Next(next.clone()))
            }

            IvrNodeType::TextToSpeech { text, language, next, .. } => {
                // TODO: Generate TTS and play via FreeSWITCH
                tracing::info!(text = %text, language = %language, "TTS playback");
                Ok(NodeResult::Next(next.clone()))
            }

            IvrNodeType::GetDigits { .. } => {
                // Wait for DTMF input from FreeSWITCH
                Ok(NodeResult::WaitForInput)
            }

            IvrNodeType::Transfer { destination, .. } => {
                // TODO: Transfer call via FreeSWITCH
                tracing::info!(destination = %destination, "Transferring call");
                Ok(NodeResult::Terminal(IvrResult::Transferred))
            }

            IvrNodeType::Hangup { cause, .. } => {
                // TODO: Hangup via FreeSWITCH
                tracing::info!(cause = %cause, "Hanging up");
                Ok(NodeResult::Terminal(IvrResult::HungUp))
            }

            IvrNodeType::CallApi { endpoint, next, .. } => {
                // TODO: Make HTTP call
                tracing::info!(endpoint = %endpoint, "Calling API");
                Ok(NodeResult::Next(next.clone()))
            }

            IvrNodeType::SetVariable { next, .. } => {
                Ok(NodeResult::Next(next.clone()))
            }

            IvrNodeType::Condition { true_node, .. } => {
                // TODO: Evaluate expression
                Ok(NodeResult::Next(true_node.clone()))
            }

            IvrNodeType::Record { next, .. } => {
                Ok(NodeResult::Next(next.clone()))
            }

            IvrNodeType::Conference { .. } => {
                Ok(NodeResult::Terminal(IvrResult::Completed))
            }

            IvrNodeType::SpeechRecognition { .. } => {
                Ok(NodeResult::WaitForInput)
            }
        }
    }

    /// Process DTMF input for a session
    pub async fn process_input(
        &self,
        session_id: &str,
        input: &str,
    ) -> Result<(), IvrError> {
        let mut session = self.sessions
            .get_mut(session_id)
            .ok_or(IvrError::SessionNotFound)?;

        session.record_input(input);

        // Get current node and determine next based on input
        let flow = self.flows
            .get(&session.flow_id)
            .ok_or(IvrError::FlowNotFound)?;

        let node = flow.get_node(&session.current_node)
            .ok_or(IvrError::NodeNotFound)?;

        if let IvrNodeType::GetDigits { branches, .. } = &node.node_type {
            if let Some(next_node) = branches.get(input).or(branches.get("default")) {
                session.move_to(next_node);
            }
        }

        Ok(())
    }
}

enum NodeResult {
    Next(String),
    Terminal(IvrResult),
    WaitForInput,
}

#[derive(Debug, thiserror::Error)]
pub enum IvrError {
    #[error("Flow not found")]
    FlowNotFound,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Node not found")]
    NodeNotFound,

    #[error("Invalid flow: {0}")]
    InvalidFlow(String),

    #[error("FreeSWITCH error: {0}")]
    FreeSwitchError(String),
}
