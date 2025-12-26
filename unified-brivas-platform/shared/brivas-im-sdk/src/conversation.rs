//! Conversation types for Instant Messaging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Conversation aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub conversation_type: ConversationType,
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub participants: Vec<Participant>,
    pub admins: Vec<Uuid>,
    pub created_by: Uuid,
    pub settings: ConversationSettings,
    pub last_message_id: Option<Uuid>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Conversation type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConversationType {
    Direct,
    Group { max_participants: u32 },
    Broadcast { subscriber_count: u64 },
}

/// Participant in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub user_id: Uuid,
    pub role: ParticipantRole,
    pub joined_at: DateTime<Utc>,
    pub muted_until: Option<DateTime<Utc>>,
    pub last_read_message_id: Option<Uuid>,
    pub notification_settings: NotificationSettings,
}

/// Participant role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Owner,
    Admin,
    Member,
}

/// Conversation settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationSettings {
    pub e2ee_enabled: bool,
    pub allow_forwarding: bool,
    pub allow_screenshots: bool,
    pub disappearing_messages_ttl: Option<u32>,
    pub only_admins_can_send: bool,
    pub only_admins_can_edit_info: bool,
}

/// Notification settings per participant
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub show_preview: bool,
    pub sound_enabled: bool,
}

impl Conversation {
    /// Create a new direct conversation
    pub fn new_direct(user1: Uuid, user2: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            conversation_type: ConversationType::Direct,
            name: None,
            description: None,
            avatar_url: None,
            participants: vec![
                Participant::new(user1, ParticipantRole::Member),
                Participant::new(user2, ParticipantRole::Member),
            ],
            admins: vec![],
            created_by: user1,
            settings: ConversationSettings::default(),
            last_message_id: None,
            last_message_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new group conversation
    pub fn new_group(name: String, creator: Uuid, members: Vec<Uuid>) -> Self {
        let now = Utc::now();
        let mut participants: Vec<Participant> = members
            .into_iter()
            .map(|u| Participant::new(u, ParticipantRole::Member))
            .collect();
        
        // Add creator as owner
        participants.insert(0, Participant::new(creator, ParticipantRole::Owner));
        
        Self {
            id: Uuid::new_v4(),
            conversation_type: ConversationType::Group { max_participants: 1000 },
            name: Some(name),
            description: None,
            avatar_url: None,
            participants,
            admins: vec![creator],
            created_by: creator,
            settings: ConversationSettings::default(),
            last_message_id: None,
            last_message_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Participant {
    pub fn new(user_id: Uuid, role: ParticipantRole) -> Self {
        Self {
            user_id,
            role,
            joined_at: Utc::now(),
            muted_until: None,
            last_read_message_id: None,
            notification_settings: NotificationSettings::default(),
        }
    }
}
