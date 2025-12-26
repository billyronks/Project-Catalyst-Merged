//! BRIVAS Instant Messaging SDK
//!
//! Protocol types and E2EE implementation for real-time messaging.

pub mod message;
pub mod conversation;
pub mod presence;
pub mod encryption;

#[cfg(test)]
mod tests;

pub use message::{Message, MessageContent, MessageType};
pub use conversation::{Conversation, ConversationType, Participant};
pub use presence::{PresenceStatus, TypingIndicator};
pub use encryption::{E2eeSession, KeyBundle};
