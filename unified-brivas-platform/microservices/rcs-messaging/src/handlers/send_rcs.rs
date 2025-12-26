//! Send RCS Message Handler

use uuid::Uuid;
use brivas_rcs_sdk::message::{RcsMessage, RcsMessageContent};
use brivas_rcs_sdk::rich_card::RichCard;
use brivas_rcs_sdk::capability::DeviceCapability;

/// Send RCS command
pub struct SendRcsCommand {
    pub agent_id: Uuid,
    pub recipient: String,
    pub content: RcsMessageContent,
    pub fallback_text: Option<String>,
}

/// Send RCS result
pub struct SendRcsResult {
    pub message_id: Uuid,
    pub channel: MessageChannel,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

pub enum MessageChannel {
    Rcs,
    SmsFallback,
}

/// Handle send RCS message
pub async fn handle(
    cmd: SendRcsCommand,
    capability: &DeviceCapability,
) -> Result<SendRcsResult, SendRcsError> {
    // Check if RCS is supported
    if !capability.rcs_enabled {
        // Fall back to SMS if enabled
        if let Some(fallback) = cmd.fallback_text {
            return Ok(SendRcsResult {
                message_id: Uuid::new_v4(),
                channel: MessageChannel::SmsFallback,
                sent_at: chrono::Utc::now(),
            });
        } else {
            return Err(SendRcsError::RcsNotSupported);
        }
    }

    // Validate content based on features
    validate_content(&cmd.content, capability)?;

    // Create message
    let message = match &cmd.content {
        RcsMessageContent::Text { text } => {
            RcsMessage::new_text(cmd.agent_id, cmd.recipient, text.clone())
        }
        RcsMessageContent::RichCard { rich_card } => {
            RcsMessage::new_rich_card(cmd.agent_id, cmd.recipient, rich_card.clone())
        }
        _ => {
            return Err(SendRcsError::UnsupportedContent);
        }
    };

    // TODO: Send via Jibe/Samsung hub
    // TODO: Store message

    Ok(SendRcsResult {
        message_id: message.id,
        channel: MessageChannel::Rcs,
        sent_at: message.created_at,
    })
}

fn validate_content(content: &RcsMessageContent, capability: &DeviceCapability) -> Result<(), SendRcsError> {
    match content {
        RcsMessageContent::RichCard { .. } => {
            if !capability.features.rich_card {
                return Err(SendRcsError::FeatureNotSupported("rich_card".to_string()));
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SendRcsError {
    #[error("RCS not supported for this recipient")]
    RcsNotSupported,
    
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),
    
    #[error("Unsupported content type")]
    UnsupportedContent,
    
    #[error("Hub delivery failed: {0}")]
    DeliveryFailed(String),
}
