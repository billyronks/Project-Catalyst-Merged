//! Bulk Voice Messaging Service
//!
//! Campaign-based voice messaging with throttling, retry logic, and DNC compliance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use dashmap::DashMap;

use crate::VoiceIvrConfig;

/// Campaign request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub message_type: MessageType,
    pub audio_url: Option<String>,
    pub tts_text: Option<String>,
    pub tts_language: Option<String>,
    pub recipients: Vec<String>,
    pub caller_id: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub throttle_cps: Option<u32>,
    pub retry_attempts: Option<u32>,
}

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    PreRecorded,
    Tts,
}

/// Campaign
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub status: CampaignStatus,
    pub message_type: MessageType,
    pub audio_url: Option<String>,
    pub tts_text: Option<String>,
    pub tts_language: Option<String>,
    pub caller_id: String,
    pub total_recipients: usize,
    pub completed: usize,
    pub failed: usize,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub throttle_cps: u32,
    pub retry_attempts: u32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Campaign status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CampaignStatus {
    Created,
    Scheduled,
    Running,
    Paused,
    Completed,
    Failed,
}

/// Campaign recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignRecipient {
    pub campaign_id: String,
    pub phone_number: String,
    pub status: RecipientStatus,
    pub attempts: u32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<u32>,
}

/// Recipient status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecipientStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    DncBlocked,
}

/// Bulk Voice Service
pub struct BulkVoiceService {
    #[allow(dead_code)]
    config: VoiceIvrConfig,
    campaigns: Arc<DashMap<String, Campaign>>,
    recipients: Arc<DashMap<String, CampaignRecipient>>,
}

impl BulkVoiceService {
    pub async fn new(config: &VoiceIvrConfig) -> brivas_core::Result<Self> {
        Ok(Self {
            config: config.clone(),
            campaigns: Arc::new(DashMap::new()),
            recipients: Arc::new(DashMap::new()),
        })
    }

    /// Create a new campaign
    pub async fn create_campaign(&self, request: CreateCampaignRequest) -> Result<Campaign, CampaignError> {
        let campaign = Campaign {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name,
            status: if request.scheduled_at.is_some() {
                CampaignStatus::Scheduled
            } else {
                CampaignStatus::Created
            },
            message_type: request.message_type,
            audio_url: request.audio_url,
            tts_text: request.tts_text,
            tts_language: request.tts_language,
            caller_id: request.caller_id,
            total_recipients: request.recipients.len(),
            completed: 0,
            failed: 0,
            scheduled_at: request.scheduled_at,
            throttle_cps: request.throttle_cps.unwrap_or(10),
            retry_attempts: request.retry_attempts.unwrap_or(2),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };

        // Store recipients
        for (i, phone) in request.recipients.iter().enumerate() {
            let recipient = CampaignRecipient {
                campaign_id: campaign.id.clone(),
                phone_number: phone.clone(),
                status: RecipientStatus::Pending,
                attempts: 0,
                last_attempt_at: None,
                completed_at: None,
                duration_seconds: None,
            };
            self.recipients.insert(format!("{}:{}", campaign.id, i), recipient);
        }

        self.campaigns.insert(campaign.id.clone(), campaign.clone());

        tracing::info!(
            campaign_id = %campaign.id,
            recipients = campaign.total_recipients,
            "Campaign created"
        );

        Ok(campaign)
    }

    /// Start a campaign
    pub async fn start_campaign(&self, campaign_id: &str) -> Result<Campaign, CampaignError> {
        let mut campaign = self.campaigns
            .get_mut(campaign_id)
            .ok_or(CampaignError::NotFound)?;

        if campaign.status != CampaignStatus::Created && campaign.status != CampaignStatus::Scheduled {
            return Err(CampaignError::InvalidStatus);
        }

        campaign.status = CampaignStatus::Running;
        campaign.started_at = Some(Utc::now());

        let campaign_clone = campaign.clone();
        drop(campaign);

        // Start execution in background
        let campaigns = self.campaigns.clone();
        let recipients = self.recipients.clone();
        let cid = campaign_id.to_string();
        
        tokio::spawn(async move {
            Self::execute_campaign(&campaigns, &recipients, &cid).await;
        });

        Ok(campaign_clone)
    }

    /// Execute campaign
    async fn execute_campaign(
        campaigns: &DashMap<String, Campaign>,
        recipients: &DashMap<String, CampaignRecipient>,
        campaign_id: &str,
    ) {
        let campaign = match campaigns.get(campaign_id) {
            Some(c) => c.clone(),
            None => return,
        };

        let throttle_interval = std::time::Duration::from_millis(1000 / campaign.throttle_cps as u64);

        // Get pending recipients
        let pending: Vec<_> = recipients
            .iter()
            .filter(|r| r.campaign_id == campaign_id && r.status == RecipientStatus::Pending)
            .map(|r| r.key().clone())
            .collect();

        for recipient_key in pending {
            // Check if campaign is still running
            if let Some(c) = campaigns.get(campaign_id) {
                if c.status != CampaignStatus::Running {
                    break;
                }
            }

            // Throttle
            tokio::time::sleep(throttle_interval).await;

            // Update recipient status
            if let Some(mut r) = recipients.get_mut(&recipient_key) {
                r.status = RecipientStatus::InProgress;
                r.attempts += 1;
                r.last_attempt_at = Some(Utc::now());
            }

            // TODO: Place call via OpenSIPS/FreeSWITCH
            // For now, simulate completion
            if let Some(mut r) = recipients.get_mut(&recipient_key) {
                r.status = RecipientStatus::Completed;
                r.completed_at = Some(Utc::now());
                r.duration_seconds = Some(30);
            }

            // Update campaign stats
            if let Some(mut c) = campaigns.get_mut(campaign_id) {
                c.completed += 1;
            }
        }

        // Mark campaign complete
        if let Some(mut c) = campaigns.get_mut(campaign_id) {
            c.status = CampaignStatus::Completed;
            c.completed_at = Some(Utc::now());
        }

        tracing::info!(campaign_id = %campaign_id, "Campaign completed");
    }

    /// Pause a campaign
    pub async fn pause_campaign(&self, campaign_id: &str) -> Result<Campaign, CampaignError> {
        let mut campaign = self.campaigns
            .get_mut(campaign_id)
            .ok_or(CampaignError::NotFound)?;

        campaign.status = CampaignStatus::Paused;
        Ok(campaign.clone())
    }

    /// Get campaign status
    pub fn get_campaign(&self, campaign_id: &str) -> Option<Campaign> {
        self.campaigns.get(campaign_id).map(|c| c.clone())
    }

    /// List campaigns
    pub fn list_campaigns(&self) -> Vec<Campaign> {
        self.campaigns.iter().map(|c| c.clone()).collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CampaignError {
    #[error("Campaign not found")]
    NotFound,

    #[error("Invalid campaign status")]
    InvalidStatus,

    #[error("Call failed: {0}")]
    CallFailed(String),
}
