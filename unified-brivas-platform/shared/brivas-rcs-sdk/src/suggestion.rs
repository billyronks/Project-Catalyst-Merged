//! Suggested actions and replies for RCS messaging

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Suggestion (either reply or action)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<SuggestedReply>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<SuggestedAction>,
}

/// Suggested reply (quick response button)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedReply {
    #[validate(length(max = 25))]
    pub text: String,
    
    #[validate(length(max = 2048))]
    pub postback_data: String,
}

/// Suggested action (opens app/browser/dialer)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedAction {
    pub text: String,
    pub postback_data: String,
    
    #[serde(flatten)]
    pub action: ActionType,
}

/// Action types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionType {
    OpenUrlAction(OpenUrlAction),
    DialAction(DialAction),
    CreateCalendarEventAction(CreateCalendarEventAction),
    ViewLocationAction(ViewLocationAction),
    ShareLocationAction(ShareLocationAction),
}

/// Open URL action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenUrlAction {
    pub url: String,
}

/// Dial phone action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialAction {
    pub phone_number: String,
}

/// Create calendar event action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCalendarEventAction {
    pub title: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
}

/// View location action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewLocationAction {
    pub lat_long: LatLong,
    pub label: Option<String>,
}

/// Share location action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareLocationAction {}

/// Latitude/Longitude coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}

impl Suggestion {
    /// Create a suggested reply
    pub fn reply(text: impl Into<String>, postback_data: impl Into<String>) -> Self {
        Self {
            reply: Some(SuggestedReply {
                text: text.into(),
                postback_data: postback_data.into(),
            }),
            action: None,
        }
    }

    /// Create an open URL action
    pub fn open_url(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            reply: None,
            action: Some(SuggestedAction {
                text: text.into(),
                postback_data: String::new(),
                action: ActionType::OpenUrlAction(OpenUrlAction { url: url.into() }),
            }),
        }
    }

    /// Create a dial action
    pub fn dial(text: impl Into<String>, phone_number: impl Into<String>) -> Self {
        Self {
            reply: None,
            action: Some(SuggestedAction {
                text: text.into(),
                postback_data: String::new(),
                action: ActionType::DialAction(DialAction { phone_number: phone_number.into() }),
            }),
        }
    }

    /// Create a view location action
    pub fn location(text: impl Into<String>, lat: f64, lng: f64, label: Option<String>) -> Self {
        Self {
            reply: None,
            action: Some(SuggestedAction {
                text: text.into(),
                postback_data: String::new(),
                action: ActionType::ViewLocationAction(ViewLocationAction {
                    lat_long: LatLong { latitude: lat, longitude: lng },
                    label,
                }),
            }),
        }
    }
}
