//! Carousel types for RCS messaging

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::rich_card::CardContent;

/// Carousel card (multiple cards in a scrollable row)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CarouselCard {
    pub card_width: CardWidth,
    
    #[validate(length(min = 2, max = 10))]
    pub card_contents: Vec<CardContent>,
}

/// Card width in carousel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CardWidth {
    Small,
    Medium,
}

impl CarouselCard {
    /// Create a new carousel with medium-width cards
    pub fn new(cards: Vec<CardContent>) -> Self {
        Self {
            card_width: CardWidth::Medium,
            card_contents: cards,
        }
    }

    /// Create a carousel with small-width cards
    pub fn small(cards: Vec<CardContent>) -> Self {
        Self {
            card_width: CardWidth::Small,
            card_contents: cards,
        }
    }

    /// Add a card to the carousel
    pub fn add_card(&mut self, card: CardContent) -> Result<(), CarouselError> {
        if self.card_contents.len() >= 10 {
            return Err(CarouselError::MaxCardsExceeded);
        }
        self.card_contents.push(card);
        Ok(())
    }
}

/// Carousel errors
#[derive(Debug, thiserror::Error)]
pub enum CarouselError {
    #[error("Maximum of 10 cards allowed in carousel")]
    MaxCardsExceeded,
    
    #[error("Minimum of 2 cards required for carousel")]
    MinCardsRequired,
}
