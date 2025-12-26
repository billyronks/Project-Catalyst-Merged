//! Rich Card Builder

use brivas_rcs_sdk::rich_card::{RichCard, StandaloneCard, CardContent, CardOrientation, Media, MediaHeight};
use brivas_rcs_sdk::carousel::CarouselCard;
use brivas_rcs_sdk::suggestion::Suggestion;

/// Builder for RCS Rich Cards
pub struct RichCardBuilder {
    title: Option<String>,
    description: Option<String>,
    media: Option<Media>,
    suggestions: Vec<Suggestion>,
    orientation: CardOrientation,
}

impl RichCardBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            media: None,
            suggestions: Vec::new(),
            orientation: CardOrientation::Vertical,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.media = Some(Media::from_url(url, MediaHeight::Medium));
        self
    }

    pub fn add_reply(mut self, text: impl Into<String>, postback: impl Into<String>) -> Self {
        self.suggestions.push(Suggestion::reply(text, postback));
        self
    }

    pub fn add_url_action(mut self, text: impl Into<String>, url: impl Into<String>) -> Self {
        self.suggestions.push(Suggestion::open_url(text, url));
        self
    }

    pub fn add_dial_action(mut self, text: impl Into<String>, phone: impl Into<String>) -> Self {
        self.suggestions.push(Suggestion::dial(text, phone));
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.orientation = CardOrientation::Horizontal;
        self
    }

    pub fn build(self) -> RichCard {
        let content = CardContent {
            title: self.title,
            description: self.description,
            media: self.media,
            suggestions: if self.suggestions.is_empty() { None } else { Some(self.suggestions) },
        };

        let standalone = match self.orientation {
            CardOrientation::Vertical => StandaloneCard::vertical(content),
            CardOrientation::Horizontal => StandaloneCard::horizontal(
                content,
                brivas_rcs_sdk::rich_card::ThumbnailAlignment::Left,
            ),
        };

        RichCard::standalone(standalone)
    }
}

impl Default for RichCardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for Carousels
pub struct CarouselBuilder {
    cards: Vec<CardContent>,
}

impl CarouselBuilder {
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }

    pub fn add_card(mut self, card: CardContent) -> Self {
        self.cards.push(card);
        self
    }

    pub fn build(self) -> Result<RichCard, &'static str> {
        if self.cards.len() < 2 {
            return Err("Carousel requires at least 2 cards");
        }
        if self.cards.len() > 10 {
            return Err("Carousel supports maximum 10 cards");
        }

        let carousel = CarouselCard::new(self.cards);
        Ok(RichCard::carousel(carousel))
    }
}

impl Default for CarouselBuilder {
    fn default() -> Self {
        Self::new()
    }
}
