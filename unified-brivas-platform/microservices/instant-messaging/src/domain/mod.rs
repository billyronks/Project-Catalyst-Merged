//! Domain module

pub mod conversation;
pub mod message;
pub mod presence;

pub use conversation::Conversation;
pub use message::Message;
pub use presence::{PresenceStatus, Status, TypingIndicator};
