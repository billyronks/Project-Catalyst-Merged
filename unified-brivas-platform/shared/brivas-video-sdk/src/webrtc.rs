//! WebRTC types

use serde::{Deserialize, Serialize};

/// SDP Offer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdpOffer {
    pub sdp: String,
    pub r#type: String,
}

impl SdpOffer {
    pub fn new(sdp: String) -> Self {
        Self {
            sdp,
            r#type: "offer".to_string(),
        }
    }
}

/// SDP Answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdpAnswer {
    pub sdp: String,
    pub r#type: String,
}

impl SdpAnswer {
    pub fn new(sdp: String) -> Self {
        Self {
            sdp,
            r#type: "answer".to_string(),
        }
    }
}

/// ICE Candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u32>,
    pub username_fragment: Option<String>,
}

/// WebRTC Session
#[derive(Debug, Clone)]
pub struct WebRtcSession {
    pub id: uuid::Uuid,
    pub local_description: Option<SdpOffer>,
    pub remote_description: Option<SdpAnswer>,
    pub ice_candidates: Vec<IceCandidate>,
    pub state: SessionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

impl WebRtcSession {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            local_description: None,
            remote_description: None,
            ice_candidates: Vec::new(),
            state: SessionState::New,
        }
    }

    pub fn add_ice_candidate(&mut self, candidate: IceCandidate) {
        self.ice_candidates.push(candidate);
    }

    pub fn set_local_description(&mut self, offer: SdpOffer) {
        self.local_description = Some(offer);
    }

    pub fn set_remote_description(&mut self, answer: SdpAnswer) {
        self.remote_description = Some(answer);
        self.state = SessionState::Connecting;
    }
}

impl Default for WebRtcSession {
    fn default() -> Self {
        Self::new()
    }
}
