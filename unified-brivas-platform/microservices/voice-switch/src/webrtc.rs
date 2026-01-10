//! WebRTC session management
//!
//! Handles WebRTC session lifecycle, SDP exchange, and ICE candidate negotiation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Error, Result};

/// WebRTC session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

/// WebRTC session
#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub state: SessionState,
    pub local_sdp: Option<String>,
    pub remote_sdp: Option<String>,
    pub local_ice_candidates: Vec<IceCandidate>,
    pub remote_ice_candidates: Vec<IceCandidate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// ICE candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
    pub username_fragment: Option<String>,
}

/// SDP payload
#[derive(Debug, Deserialize)]
pub struct SdpPayload {
    pub sdp: String,
}

/// Request to create a WebRTC session
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: Uuid,
    pub offer_sdp: Option<String>,
}

/// Create a new WebRTC session
pub async fn create_session(
    db: &brivas_lumadb::LumaDbPool,
    req: CreateSessionRequest,
) -> Result<Session> {
    let session_id = Uuid::new_v4();
    let now = Utc::now();

    let client = db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    client
        .execute(
            r#"
            INSERT INTO webrtc_sessions (
                session_id, user_id, state, local_sdp, remote_sdp,
                local_ice_candidates, remote_ice_candidates, created_at, updated_at
            ) VALUES ($1, $2, 'new', $3, NULL, '[]', '[]', $4, $4)
            "#,
            &[&session_id, &req.user_id, &req.offer_sdp, &now],
        )
        .await?;

    Ok(Session {
        session_id,
        user_id: req.user_id,
        state: SessionState::New,
        local_sdp: req.offer_sdp,
        remote_sdp: None,
        local_ice_candidates: vec![],
        remote_ice_candidates: vec![],
        created_at: now,
        updated_at: now,
    })
}

/// Get a WebRTC session by ID
pub async fn get_session(db: &brivas_lumadb::LumaDbPool, session_id: Uuid) -> Result<Session> {
    let client = db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    let row = client
        .query_opt(
            "SELECT * FROM webrtc_sessions WHERE session_id = $1",
            &[&session_id],
        )
        .await?
        .ok_or_else(|| Error::Internal(format!("Session not found: {}", session_id)))?;

    Ok(Session {
        session_id: row.get("session_id"),
        user_id: row.get("user_id"),
        state: serde_json::from_str(row.get("state")).unwrap_or(SessionState::New),
        local_sdp: row.get("local_sdp"),
        remote_sdp: row.get("remote_sdp"),
        local_ice_candidates: serde_json::from_str(row.get("local_ice_candidates"))
            .unwrap_or_default(),
        remote_ice_candidates: serde_json::from_str(row.get("remote_ice_candidates"))
            .unwrap_or_default(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

/// Delete a WebRTC session
pub async fn delete_session(db: &brivas_lumadb::LumaDbPool, session_id: Uuid) -> Result<()> {
    let client = db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    client
        .execute(
            "DELETE FROM webrtc_sessions WHERE session_id = $1",
            &[&session_id],
        )
        .await?;

    Ok(())
}

/// Set local SDP
pub async fn set_local_sdp(
    db: &brivas_lumadb::LumaDbPool,
    session_id: Uuid,
    sdp: &str,
) -> Result<()> {
    let client = db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    client
        .execute(
            "UPDATE webrtc_sessions SET local_sdp = $2, updated_at = NOW() WHERE session_id = $1",
            &[&session_id, &sdp],
        )
        .await?;

    Ok(())
}

/// Set remote SDP
pub async fn set_remote_sdp(
    db: &brivas_lumadb::LumaDbPool,
    session_id: Uuid,
    sdp: &str,
) -> Result<()> {
    let client = db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    client
        .execute(
            "UPDATE webrtc_sessions SET remote_sdp = $2, state = 'connecting', updated_at = NOW() WHERE session_id = $1",
            &[&session_id, &sdp],
        )
        .await?;

    Ok(())
}

/// Add ICE candidate
pub async fn add_ice_candidate(
    db: &brivas_lumadb::LumaDbPool,
    session_id: Uuid,
    candidate: &IceCandidate,
    is_local: bool,
) -> Result<()> {
    let client = db.get().await.map_err(|e| Error::Internal(e.to_string()))?;

    let column = if is_local {
        "local_ice_candidates"
    } else {
        "remote_ice_candidates"
    };

    let candidate_json = serde_json::to_string(candidate)
        .map_err(|e| Error::Internal(e.to_string()))?;

    client
        .execute(
            &format!(
                "UPDATE webrtc_sessions SET {} = {} || $2::jsonb, updated_at = NOW() WHERE session_id = $1",
                column, column
            ),
            &[&session_id, &candidate_json],
        )
        .await?;

    Ok(())
}
