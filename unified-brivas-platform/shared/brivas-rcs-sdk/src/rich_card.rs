//! Rich Card types for RCS messaging

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Rich Card container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichCard {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standalone_card: Option<StandaloneCard>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carousel_card: Option<super::carousel::CarouselCard>,
}

/// Standalone rich card
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct StandaloneCard {
    pub card_orientation: CardOrientation,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_image_alignment: Option<ThumbnailAlignment>,
    
    pub card_content: CardContent,
}

/// Card orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CardOrientation {
    Vertical,
    Horizontal,
}

/// Thumbnail alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThumbnailAlignment {
    Left,
    Right,
}

/// Card content
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CardContent {
    #[validate(length(max = 200))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    #[validate(length(max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<Media>,
    
    #[validate(length(max = 4))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<super::suggestion::Suggestion>>,
}

/// Media content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub height: MediaHeight,
    pub content_info: ContentInfo,
}

/// Media height
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaHeight {
    Short,
    Medium,
    Tall,
}

/// Content info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentInfo {
    pub file_url: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_refresh: Option<bool>,
}

impl RichCard {
    /// Create a standalone rich card
    pub fn standalone(card: StandaloneCard) -> Self {
        Self {
            standalone_card: Some(card),
            carousel_card: None,
        }
    }

    /// Create a carousel rich card
    pub fn carousel(carousel: super::carousel::CarouselCard) -> Self {
        Self {
            standalone_card: None,
            carousel_card: Some(carousel),
        }
    }
}

impl StandaloneCard {
    /// Create a vertical card with content
    pub fn vertical(content: CardContent) -> Self {
        Self {
            card_orientation: CardOrientation::Vertical,
            thumbnail_image_alignment: None,
            card_content: content,
        }
    }

    /// Create a horizontal card with content
    pub fn horizontal(content: CardContent, thumbnail_alignment: ThumbnailAlignment) -> Self {
        Self {
            card_orientation: CardOrientation::Horizontal,
            thumbnail_image_alignment: Some(thumbnail_alignment),
            card_content: content,
        }
    }
}

impl CardContent {
    /// Create card content with title and description
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            description: Some(description.into()),
            media: None,
            suggestions: None,
        }
    }

    /// Add media to the card
    pub fn with_media(mut self, media: Media) -> Self {
        self.media = Some(media);
        self
    }

    /// Add suggestions to the card
    pub fn with_suggestions(mut self, suggestions: Vec<super::suggestion::Suggestion>) -> Self {
        self.suggestions = Some(suggestions);
        self
    }
}

impl Media {
    /// Create media from URL
    pub fn from_url(url: impl Into<String>, height: MediaHeight) -> Self {
        Self {
            height,
            content_info: ContentInfo {
                file_url: url.into(),
                thumbnail_url: None,
                force_refresh: None,
            },
        }
    }
}
