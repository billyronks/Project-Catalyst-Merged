//! BRIVAS RCS Messaging SDK
//!
//! Rich Communication Services protocol types for A2P/P2P messaging.

pub mod agent;
pub mod message;
pub mod rich_card;
pub mod carousel;
pub mod suggestion;
pub mod capability;

#[cfg(test)]
mod tests;

pub use agent::{RcsAgent, AgentVerificationStatus};
pub use message::{RcsMessage, RcsMessageStatus};
pub use rich_card::{RichCard, StandaloneCard, CardContent, Media};
pub use carousel::CarouselCard;
pub use suggestion::{Suggestion, SuggestedReply, SuggestedAction};
pub use capability::{DeviceCapability, RcsFeatures};
