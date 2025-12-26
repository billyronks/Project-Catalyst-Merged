//! Unified message model abstraction across 16 platforms

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Supported messaging platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    WhatsApp,
    FacebookMessenger,
    Telegram,
    WeChat,
    Snapchat,
    Signal,
    Viber,
    Line,
    Discord,
    IMessage,
    QQ,
    Zalo,
    KakaoTalk,
    Slack,
    MicrosoftTeams,
    GoogleChat,
}

impl Platform {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "whatsapp" => Self::WhatsApp,
            "facebook" | "messenger" => Self::FacebookMessenger,
            "telegram" => Self::Telegram,
            "wechat" => Self::WeChat,
            "snapchat" => Self::Snapchat,
            "signal" => Self::Signal,
            "viber" => Self::Viber,
            "line" => Self::Line,
            "discord" => Self::Discord,
            "imessage" => Self::IMessage,
            "qq" => Self::QQ,
            "zalo" => Self::Zalo,
            "kakaotalk" => Self::KakaoTalk,
            "slack" => Self::Slack,
            "teams" => Self::MicrosoftTeams,
            "googlechat" => Self::GoogleChat,
            _ => Self::WhatsApp,
        }
    }
}

/// Unified message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub id: String,
    pub conversation_id: String,
    pub platform: Platform,
    pub direction: MessageDirection,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: MessageContent,
    pub reply_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub status: MessageStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
}

/// Message content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    Text { body: String },
    Image { url: String, caption: Option<String> },
    Video { url: String, caption: Option<String> },
    Audio { url: String },
    Document { url: String, filename: String },
    Location { latitude: f64, longitude: f64, name: Option<String> },
    Contact { contacts: Vec<ContactCard> },
    Interactive { interactive_type: InteractiveType, body: String, action: InteractiveAction },
    Template { template_name: String, language: String, components: Vec<TemplateComponent> },
    Sticker { sticker_id: String },
    Reaction { emoji: String, message_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCard {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractiveType {
    Button,
    List,
    Carousel,
    QuickReply,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveAction {
    pub buttons: Option<Vec<Button>>,
    pub sections: Option<Vec<ListSection>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Button {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSection {
    pub title: String,
    pub rows: Vec<ListRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRow {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateComponent {
    pub component_type: String,
    pub parameters: Vec<TemplateParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub param_type: String,
    pub text: Option<String>,
    pub image: Option<MediaObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaObject {
    pub link: String,
}

/// Conversation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub platform: Platform,
    pub platform_conversation_id: String,
    pub participants: Vec<Participant>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub platform_id: String,
    pub name: Option<String>,
    pub role: ParticipantRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    User,
    Agent,
    Bot,
}
