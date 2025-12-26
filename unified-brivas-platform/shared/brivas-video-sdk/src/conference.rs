//! Conference types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Video conference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conference {
    pub id: Uuid,
    pub name: String,
    pub host_id: Uuid,
    pub conference_type: ConferenceType,
    pub settings: ConferenceSettings,
    pub layout: ConferenceLayout,
    pub state: ConferenceState,
    pub participants: Vec<Participant>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub join_url: String,
    pub dial_in_number: Option<String>,
    pub access_code: Option<String>,
}

impl Conference {
    pub fn new(name: String, host_id: Uuid) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            name,
            host_id,
            conference_type: ConferenceType::Meeting,
            settings: ConferenceSettings::default(),
            layout: ConferenceLayout::Auto,
            state: ConferenceState::Created,
            participants: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
            join_url: format!("https://meet.brivas.io/{}", id),
            dial_in_number: None,
            access_code: None,
        }
    }

    pub fn add_participant(&mut self, participant: Participant) {
        self.participants.push(participant);
        if self.state == ConferenceState::Created {
            self.state = ConferenceState::Active;
            self.started_at = Some(Utc::now());
        }
    }

    pub fn remove_participant(&mut self, participant_id: Uuid) {
        self.participants.retain(|p| p.id != participant_id);
        if self.participants.is_empty() {
            self.state = ConferenceState::Ended;
            self.ended_at = Some(Utc::now());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConferenceType {
    Meeting,
    Webinar,
    Broadcast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConferenceState {
    Created,
    Waiting,
    Active,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConferenceLayout {
    Auto,
    Grid,
    SpeakerFocus,
    Presentation,
    Gallery,
    Sidebar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceSettings {
    pub max_participants: u32,
    pub mute_on_entry: bool,
    pub video_on_entry: bool,
    pub waiting_room: bool,
    pub recording_enabled: bool,
    pub transcription_enabled: bool,
    pub chat_enabled: bool,
    pub screen_share_enabled: bool,
    pub reactions_enabled: bool,
    pub max_quality: crate::VideoQuality,
    pub password: Option<String>,
    pub require_authentication: bool,
}

impl Default for ConferenceSettings {
    fn default() -> Self {
        Self {
            max_participants: 100,
            mute_on_entry: false,
            video_on_entry: true,
            waiting_room: false,
            recording_enabled: true,
            transcription_enabled: false,
            chat_enabled: true,
            screen_share_enabled: true,
            reactions_enabled: true,
            max_quality: crate::VideoQuality::Hd,
            password: None,
            require_authentication: false,
        }
    }
}

/// Conference participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: ParticipantRole,
    pub state: ParticipantState,
    pub audio_enabled: bool,
    pub video_enabled: bool,
    pub screen_sharing: bool,
    pub hand_raised: bool,
    pub connection_quality: ConnectionQuality,
    pub video_quality: crate::VideoQuality,
    pub joined_at: DateTime<Utc>,
}

impl Participant {
    pub fn new(user_id: Uuid, display_name: String, role: ParticipantRole) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            display_name,
            avatar_url: None,
            role,
            state: ParticipantState::Joining,
            audio_enabled: true,
            video_enabled: true,
            screen_sharing: false,
            hand_raised: false,
            connection_quality: ConnectionQuality::Good,
            video_quality: crate::VideoQuality::Auto,
            joined_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantRole {
    Attendee,
    Presenter,
    Moderator,
    Host,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantState {
    Joining,
    Connected,
    Reconnecting,
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Unknown,
}
