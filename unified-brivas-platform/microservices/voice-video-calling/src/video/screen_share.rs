//! Screen Sharing
//!
//! Handles screen and application sharing in video calls/conferences.

use uuid::Uuid;

/// Screen share session
pub struct ScreenShareSession {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub participant_id: Uuid,
    pub share_type: ShareType,
    pub state: ShareState,
}

#[derive(Debug, Clone, Copy)]
pub enum ShareType {
    Screen,
    Window,
    Tab,
}

#[derive(Debug, Clone, Copy)]
pub enum ShareState {
    Starting,
    Active,
    Paused,
    Stopped,
}

/// Screen share manager
pub struct ScreenShareManager {
    // Active screen share sessions
}

impl ScreenShareManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Start screen sharing
    pub async fn start_share(
        &self,
        conference_id: Uuid,
        participant_id: Uuid,
        share_type: ShareType,
    ) -> Result<ScreenShareSession, ScreenShareError> {
        let session = ScreenShareSession {
            id: Uuid::new_v4(),
            conference_id,
            participant_id,
            share_type,
            state: ShareState::Starting,
        };

        // TODO: Notify media server and update conference layout
        tracing::info!(
            session_id = %session.id,
            conference_id = %conference_id,
            participant_id = %participant_id,
            "Starting screen share"
        );

        Ok(session)
    }

    /// Stop screen sharing
    pub async fn stop_share(&self, session_id: Uuid) -> Result<(), ScreenShareError> {
        tracing::info!(session_id = %session_id, "Stopping screen share");
        Ok(())
    }

    /// Pause screen sharing
    pub async fn pause_share(&self, session_id: Uuid) -> Result<(), ScreenShareError> {
        tracing::info!(session_id = %session_id, "Pausing screen share");
        Ok(())
    }

    /// Resume screen sharing
    pub async fn resume_share(&self, session_id: Uuid) -> Result<(), ScreenShareError> {
        tracing::info!(session_id = %session_id, "Resuming screen share");
        Ok(())
    }
}

impl Default for ScreenShareManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScreenShareError {
    #[error("Session not found")]
    NotFound,
    #[error("Already sharing")]
    AlreadySharing,
    #[error("Not sharing")]
    NotSharing,
}
