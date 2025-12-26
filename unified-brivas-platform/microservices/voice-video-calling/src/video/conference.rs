//! Video Conference Management
//!
//! Manages video conference rooms, participants, and layouts.

use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use tokio::sync::broadcast;
use brivas_video_sdk::{
    Conference, ConferenceSettings, ConferenceLayout, ConferenceType, ConferenceState,
    Participant, ParticipantRole, ParticipantState,
};

/// Conference Manager
pub struct ConferenceManager {
    /// Active conferences
    conferences: Arc<DashMap<Uuid, Conference>>,
    /// Event broadcaster
    event_tx: broadcast::Sender<ConferenceEvent>,
}

#[derive(Clone, Debug)]
pub enum ConferenceEvent {
    ConferenceCreated { conference_id: Uuid },
    ParticipantJoined { conference_id: Uuid, participant_id: Uuid },
    ParticipantLeft { conference_id: Uuid, participant_id: Uuid },
    LayoutChanged { conference_id: Uuid, layout: String },
    RecordingStarted { conference_id: Uuid },
    RecordingStopped { conference_id: Uuid },
    ConferenceEnded { conference_id: Uuid },
}

impl ConferenceManager {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            conferences: Arc::new(DashMap::new()),
            event_tx,
        }
    }

    /// Create a new conference
    pub async fn create_conference(
        &self,
        name: String,
        host_id: Uuid,
        settings: Option<ConferenceSettings>,
    ) -> Conference {
        let mut conference = Conference::new(name, host_id);
        
        if let Some(s) = settings {
            conference.settings = s;
        }

        let conference_id = conference.id;
        self.conferences.insert(conference_id, conference.clone());

        let _ = self.event_tx.send(ConferenceEvent::ConferenceCreated { conference_id });

        conference
    }

    /// Get a conference by ID
    pub fn get_conference(&self, conference_id: Uuid) -> Option<Conference> {
        self.conferences.get(&conference_id).map(|c| c.clone())
    }

    /// Join a conference
    pub async fn join_conference(
        &self,
        conference_id: Uuid,
        user_id: Uuid,
        display_name: String,
        role: ParticipantRole,
    ) -> Result<Participant, ConferenceError> {
        let mut conference = self.conferences
            .get_mut(&conference_id)
            .ok_or(ConferenceError::NotFound)?;

        // Check max participants
        if conference.participants.len() >= conference.settings.max_participants as usize {
            return Err(ConferenceError::MaxParticipantsReached);
        }

        let participant = Participant::new(user_id, display_name, role);
        let participant_id = participant.id;
        
        conference.add_participant(participant.clone());

        let _ = self.event_tx.send(ConferenceEvent::ParticipantJoined {
            conference_id,
            participant_id,
        });

        Ok(participant)
    }

    /// Leave a conference
    pub async fn leave_conference(
        &self,
        conference_id: Uuid,
        participant_id: Uuid,
    ) -> Result<(), ConferenceError> {
        let mut conference = self.conferences
            .get_mut(&conference_id)
            .ok_or(ConferenceError::NotFound)?;

        conference.remove_participant(participant_id);

        let _ = self.event_tx.send(ConferenceEvent::ParticipantLeft {
            conference_id,
            participant_id,
        });

        // Clean up empty conferences
        if conference.participants.is_empty() {
            drop(conference);
            self.conferences.remove(&conference_id);
            let _ = self.event_tx.send(ConferenceEvent::ConferenceEnded { conference_id });
        }

        Ok(())
    }

    /// Update conference layout
    pub async fn set_layout(
        &self,
        conference_id: Uuid,
        layout: ConferenceLayout,
    ) -> Result<(), ConferenceError> {
        let mut conference = self.conferences
            .get_mut(&conference_id)
            .ok_or(ConferenceError::NotFound)?;

        conference.layout = layout;

        let _ = self.event_tx.send(ConferenceEvent::LayoutChanged {
            conference_id,
            layout: format!("{:?}", layout),
        });

        Ok(())
    }

    /// Mute all participants
    pub async fn mute_all(&self, conference_id: Uuid) -> Result<(), ConferenceError> {
        let mut conference = self.conferences
            .get_mut(&conference_id)
            .ok_or(ConferenceError::NotFound)?;

        for participant in &mut conference.participants {
            participant.audio_enabled = false;
        }

        Ok(())
    }

    /// Get list of participants
    pub fn get_participants(&self, conference_id: Uuid) -> Result<Vec<Participant>, ConferenceError> {
        let conference = self.conferences
            .get(&conference_id)
            .ok_or(ConferenceError::NotFound)?;

        Ok(conference.participants.clone())
    }

    /// Subscribe to conference events
    pub fn subscribe(&self) -> broadcast::Receiver<ConferenceEvent> {
        self.event_tx.subscribe()
    }
}

impl Default for ConferenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConferenceError {
    #[error("Conference not found")]
    NotFound,
    #[error("Maximum participants reached")]
    MaxParticipantsReached,
    #[error("Not authorized")]
    NotAuthorized,
    #[error("Conference ended")]
    Ended,
}
