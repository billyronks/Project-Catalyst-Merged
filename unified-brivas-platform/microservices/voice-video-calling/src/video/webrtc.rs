//! WebRTC Gateway
//!
//! Handles WebRTC signaling and media negotiation for browser clients.

use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use tokio::sync::broadcast;
use brivas_video_sdk::{WebRtcSession, SdpOffer, SdpAnswer, IceCandidate};

/// WebRTC Gateway for handling browser-based video calls
pub struct WebRtcGateway {
    /// Active sessions by session ID
    sessions: Arc<DashMap<Uuid, WebRtcSession>>,
    /// TURN server credentials
    turn_server: String,
    /// Event broadcaster
    event_tx: broadcast::Sender<WebRtcEvent>,
}

#[derive(Clone, Debug)]
pub enum WebRtcEvent {
    SessionCreated { session_id: Uuid },
    SdpOffer { session_id: Uuid, offer: String },
    SdpAnswer { session_id: Uuid, answer: String },
    IceCandidate { session_id: Uuid, candidate: String },
    SessionClosed { session_id: Uuid },
}

impl WebRtcGateway {
    pub fn new(turn_server: &str) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            sessions: Arc::new(DashMap::new()),
            turn_server: turn_server.to_string(),
            event_tx,
        }
    }

    /// Create a new WebRTC session
    pub async fn create_session(&self) -> Uuid {
        let session = WebRtcSession::new();
        let session_id = session.id;
        self.sessions.insert(session_id, session);
        
        let _ = self.event_tx.send(WebRtcEvent::SessionCreated { session_id });
        
        session_id
    }

    /// Handle SDP offer from client
    pub async fn handle_offer(
        &self,
        session_id: Uuid,
        offer: SdpOffer,
    ) -> Result<SdpAnswer, WebRtcError> {
        let mut session = self.sessions
            .get_mut(&session_id)
            .ok_or(WebRtcError::SessionNotFound)?;

        session.set_local_description(offer);

        // TODO: Process offer through media server (Janus/FreeSWITCH)
        // For now, return a placeholder answer
        let answer = SdpAnswer::new("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\n...".to_string());

        let _ = self.event_tx.send(WebRtcEvent::SdpAnswer {
            session_id,
            answer: answer.sdp.clone(),
        });

        Ok(answer)
    }

    /// Add ICE candidate
    pub async fn add_ice_candidate(
        &self,
        session_id: Uuid,
        candidate: IceCandidate,
    ) -> Result<(), WebRtcError> {
        let mut session = self.sessions
            .get_mut(&session_id)
            .ok_or(WebRtcError::SessionNotFound)?;

        session.add_ice_candidate(candidate.clone());

        let _ = self.event_tx.send(WebRtcEvent::IceCandidate {
            session_id,
            candidate: candidate.candidate,
        });

        Ok(())
    }

    /// Close a session
    pub async fn close_session(&self, session_id: Uuid) -> Result<(), WebRtcError> {
        self.sessions.remove(&session_id);
        let _ = self.event_tx.send(WebRtcEvent::SessionClosed { session_id });
        Ok(())
    }

    /// Get TURN credentials
    pub fn get_turn_credentials(&self) -> TurnCredentials {
        // TODO: Generate ephemeral TURN credentials
        TurnCredentials {
            server: self.turn_server.clone(),
            username: format!("turn-user-{}", Uuid::new_v4()),
            password: "generated-password".to_string(),
            ttl: 86400,
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<WebRtcEvent> {
        self.event_tx.subscribe()
    }
}

pub struct TurnCredentials {
    pub server: String,
    pub username: String,
    pub password: String,
    pub ttl: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum WebRtcError {
    #[error("Session not found")]
    SessionNotFound,
    #[error("Invalid SDP: {0}")]
    InvalidSdp(String),
    #[error("Media server error: {0}")]
    MediaServerError(String),
}
