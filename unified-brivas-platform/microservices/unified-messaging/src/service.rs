//! Messaging Hub Service

use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::{PlatformAdapter, TelegramAdapter, WhatsAppAdapter};
use crate::model::{MessageContent, MessageDirection, MessageStatus, Platform, UnifiedMessage};

/// Messaging Hub Service orchestrating all platform adapters
#[derive(Clone)]
pub struct MessagingHubService {
    adapters: Arc<HashMap<Platform, Arc<dyn PlatformAdapter>>>,
}

impl MessagingHubService {
    pub async fn new() -> brivas_core::Result<Self> {
        let mut adapters: HashMap<Platform, Arc<dyn PlatformAdapter>> = HashMap::new();

        // Initialize WhatsApp adapter if configured
        if let (Ok(phone_id), Ok(token)) = (
            std::env::var("WHATSAPP_PHONE_NUMBER_ID"),
            std::env::var("WHATSAPP_ACCESS_TOKEN"),
        ) {
            adapters.insert(
                Platform::WhatsApp,
                Arc::new(WhatsAppAdapter::new(phone_id, token)),
            );
            tracing::info!("WhatsApp adapter initialized");
        }

        // Initialize Telegram adapter if configured
        if let Ok(token) = std::env::var("TELEGRAM_BOT_TOKEN") {
            adapters.insert(Platform::Telegram, Arc::new(TelegramAdapter::new(token)));
            tracing::info!("Telegram adapter initialized");
        }

        Ok(Self {
            adapters: Arc::new(adapters),
        })
    }

    /// Get list of active platforms
    pub async fn active_platforms(&self) -> Vec<Platform> {
        self.adapters.keys().copied().collect()
    }

    /// Send message to a platform
    pub async fn send_message(
        &self,
        platform: Platform,
        recipient_id: &str,
        content: MessageContent,
    ) -> brivas_core::Result<(String, String)> {
        let adapter = self
            .adapters
            .get(&platform)
            .ok_or_else(|| brivas_core::BrivasError::NotFound(format!("Platform {:?} not configured", platform)))?;

        let message = UnifiedMessage {
            id: uuid::Uuid::new_v4().to_string(),
            conversation_id: String::new(),
            platform,
            direction: MessageDirection::Outbound,
            sender_id: "system".to_string(),
            recipient_id: recipient_id.to_string(),
            content,
            reply_to: None,
            created_at: chrono::Utc::now(),
            delivered_at: None,
            read_at: None,
            status: MessageStatus::Pending,
        };

        let platform_id = adapter
            .send_message(&message)
            .await
            .map_err(|e| brivas_core::BrivasError::Network(e.to_string()))?;

        Ok((message.id, platform_id.0))
    }

    /// Get platform capabilities
    pub fn get_capabilities(&self, platform: Platform) -> Option<crate::adapters::PlatformCapabilities> {
        self.adapters.get(&platform).map(|a| a.capabilities())
    }
}
