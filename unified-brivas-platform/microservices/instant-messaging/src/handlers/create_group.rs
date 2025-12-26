//! Create group handler

use uuid::Uuid;
use brivas_im_sdk::conversation::{Conversation, ConversationSettings};

/// Create group command
pub struct CreateGroupCommand {
    pub creator_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<Uuid>,
    pub settings: Option<ConversationSettings>,
}

/// Create group result
pub struct CreateGroupResult {
    pub conversation_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Handle create group
pub async fn handle(cmd: CreateGroupCommand) -> Result<CreateGroupResult, CreateGroupError> {
    // Validate
    if cmd.name.is_empty() {
        return Err(CreateGroupError::InvalidName);
    }
    if cmd.name.len() > 100 {
        return Err(CreateGroupError::NameTooLong);
    }
    if cmd.members.len() > 1000 {
        return Err(CreateGroupError::TooManyMembers);
    }
    
    // Create conversation
    let conversation = Conversation::new_group(
        cmd.name,
        cmd.creator_id,
        cmd.members,
    );
    
    // TODO: Store conversation
    // TODO: Send system message about creation
    // TODO: Notify members
    
    Ok(CreateGroupResult {
        conversation_id: conversation.id,
        created_at: conversation.created_at,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum CreateGroupError {
    #[error("Group name is required")]
    InvalidName,
    
    #[error("Group name too long (max 100 chars)")]
    NameTooLong,
    
    #[error("Too many members (max 1000)")]
    TooManyMembers,
    
    #[error("Storage error: {0}")]
    StorageError(String),
}
