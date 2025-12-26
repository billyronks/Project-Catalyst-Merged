//! Send message handler

use uuid::Uuid;
use brivas_im_sdk::message::{Message, MessageContent};

/// Send message command
pub struct SendMessageCommand {
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: MessageContent,
    pub reply_to: Option<Uuid>,
    pub encrypted: bool,
}

/// Send message result
pub struct SendMessageResult {
    pub message_id: Uuid,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

/// Handle send message
pub async fn handle(cmd: SendMessageCommand) -> Result<SendMessageResult, SendMessageError> {
    // Validate content
    validate_content(&cmd.content)?;
    
    // Create message
    let message = Message::new_text(
        cmd.conversation_id,
        cmd.sender_id,
        match &cmd.content {
            MessageContent::Text { text, .. } => text.clone(),
            _ => String::new(),
        },
    );
    
    // TODO: Store message
    // TODO: Publish to WebSocket subscribers
    // TODO: Send push notifications
    
    Ok(SendMessageResult {
        message_id: message.id,
        sent_at: message.created_at,
    })
}

fn validate_content(content: &MessageContent) -> Result<(), SendMessageError> {
    match content {
        MessageContent::Text { text, .. } => {
            if text.is_empty() {
                return Err(SendMessageError::EmptyContent);
            }
            if text.len() > 10000 {
                return Err(SendMessageError::ContentTooLarge);
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SendMessageError {
    #[error("Message content is empty")]
    EmptyContent,
    
    #[error("Message content too large")]
    ContentTooLarge,
    
    #[error("Conversation not found")]
    ConversationNotFound,
    
    #[error("Not a participant")]
    NotParticipant,
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
}
