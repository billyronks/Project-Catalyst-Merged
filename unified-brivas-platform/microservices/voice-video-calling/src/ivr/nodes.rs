//! IVR Node Types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IVR Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrNode {
    pub id: String,
    pub node_type: IvrNodeType,
}

/// IVR Node Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IvrNodeType {
    /// Play pre-recorded audio
    PlayAudio {
        audio_url: String,
        next: String,
    },

    /// Text-to-speech
    TextToSpeech {
        text: String,
        language: String,
        voice: Option<String>,
        next: String,
    },

    /// Collect DTMF digits
    GetDigits {
        prompt_audio: Option<String>,
        prompt_tts: Option<String>,
        num_digits: u8,
        timeout_ms: u32,
        max_attempts: u8,
        branches: HashMap<String, String>,  // digit -> node_id
    },

    /// Speech recognition
    SpeechRecognition {
        prompt_audio: Option<String>,
        prompt_tts: Option<String>,
        grammar: Option<String>,
        timeout_ms: u32,
        branches: HashMap<String, String>,  // phrase -> node_id
    },

    /// Transfer call
    Transfer {
        destination: String,
        transfer_type: TransferType,
        announce_audio: Option<String>,
    },

    /// Hang up call
    Hangup {
        cause: String,
        goodbye_audio: Option<String>,
    },

    /// Call external API
    CallApi {
        endpoint: String,
        method: String,
        body_template: Option<String>,
        response_variable: String,
        next: String,
        error_node: Option<String>,
    },

    /// Set variable
    SetVariable {
        variable: String,
        value: String,
        next: String,
    },

    /// Conditional branch
    Condition {
        expression: String,
        true_node: String,
        false_node: String,
    },

    /// Record audio
    Record {
        max_duration_seconds: u32,
        beep: bool,
        silence_threshold: u32,
        variable: String,
        next: String,
    },

    /// Conference bridge
    Conference {
        room_id: String,
        muted: bool,
        announce_join: bool,
    },
}

/// Transfer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    Blind,
    Attended,
    Consult,
}

impl IvrNode {
    /// Create a new play audio node
    pub fn play_audio(id: &str, audio_url: &str, next: &str) -> Self {
        Self {
            id: id.to_string(),
            node_type: IvrNodeType::PlayAudio {
                audio_url: audio_url.to_string(),
                next: next.to_string(),
            },
        }
    }

    /// Create a new TTS node
    pub fn tts(id: &str, text: &str, language: &str, next: &str) -> Self {
        Self {
            id: id.to_string(),
            node_type: IvrNodeType::TextToSpeech {
                text: text.to_string(),
                language: language.to_string(),
                voice: None,
                next: next.to_string(),
            },
        }
    }

    /// Create a menu (get digits) node
    pub fn menu(id: &str, prompt: &str, branches: HashMap<String, String>) -> Self {
        Self {
            id: id.to_string(),
            node_type: IvrNodeType::GetDigits {
                prompt_audio: None,
                prompt_tts: Some(prompt.to_string()),
                num_digits: 1,
                timeout_ms: 5000,
                max_attempts: 3,
                branches,
            },
        }
    }

    /// Create a hangup node
    pub fn hangup(id: &str, cause: &str) -> Self {
        Self {
            id: id.to_string(),
            node_type: IvrNodeType::Hangup {
                cause: cause.to_string(),
                goodbye_audio: None,
            },
        }
    }

    /// Get all next node IDs this node can transition to
    pub fn get_next_nodes(&self) -> Vec<String> {
        match &self.node_type {
            IvrNodeType::PlayAudio { next, .. } => vec![next.clone()],
            IvrNodeType::TextToSpeech { next, .. } => vec![next.clone()],
            IvrNodeType::GetDigits { branches, .. } => branches.values().cloned().collect(),
            IvrNodeType::SpeechRecognition { branches, .. } => branches.values().cloned().collect(),
            IvrNodeType::Transfer { .. } => vec![],  // Terminal
            IvrNodeType::Hangup { .. } => vec![],  // Terminal
            IvrNodeType::CallApi { next, error_node, .. } => {
                let mut nodes = vec![next.clone()];
                if let Some(err) = error_node {
                    nodes.push(err.clone());
                }
                nodes
            }
            IvrNodeType::SetVariable { next, .. } => vec![next.clone()],
            IvrNodeType::Condition { true_node, false_node, .. } => {
                vec![true_node.clone(), false_node.clone()]
            }
            IvrNodeType::Record { next, .. } => vec![next.clone()],
            IvrNodeType::Conference { .. } => vec![],  // Terminal (until exit)
        }
    }
}
