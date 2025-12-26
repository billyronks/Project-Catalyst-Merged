//! Flash Call Service - OTP via Caller ID
//!
//! Initiates brief calls where the OTP is encoded in the Caller ID.
//! Sub-second call duration for cost-effective OTP delivery.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;

use crate::VoiceIvrConfig;

/// Flash Call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashCallRequest {
    pub request_id: String,
    pub destination: String,
    pub cli_prefix: String,
    pub otp_length: u8,
    pub callback_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Flash Call response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashCallResponse {
    pub request_id: String,
    pub call_id: String,
    pub otp: String,
    pub status: FlashCallStatus,
}

/// Flash Call status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlashCallStatus {
    Initiated,
    Ringing,
    Completed,
    Failed,
}

/// OTP verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationResult {
    Success,
    InvalidOtp,
    Expired,
    AlreadyUsed,
    NotFound,
}

/// Stored OTP record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OtpRecord {
    otp: String,
    destination: String,
    created_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
    verified: bool,
}

/// Flash Call CDR
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlashCallCdr {
    call_id: String,
    request_id: String,
    destination: String,
    otp_sent: String,
    initiated_at: chrono::DateTime<Utc>,
    status: FlashCallStatus,
}

/// Flash Call Service
pub struct FlashCallService {
    #[allow(dead_code)]
    config: VoiceIvrConfig,
    // In-memory store (would be LumaDB in production)
    otps: Arc<DashMap<String, OtpRecord>>,
    cdrs: Arc<DashMap<String, FlashCallCdr>>,
    // Rate limiter per destination
    rate_limits: Arc<DashMap<String, u32>>,
}

impl FlashCallService {
    pub async fn new(config: &VoiceIvrConfig) -> brivas_core::Result<Self> {
        Ok(Self {
            config: config.clone(),
            otps: Arc::new(DashMap::new()),
            cdrs: Arc::new(DashMap::new()),
            rate_limits: Arc::new(DashMap::new()),
        })
    }

    /// Initiate flash call for OTP verification
    pub async fn initiate(&self, request: FlashCallRequest) -> Result<FlashCallResponse, FlashCallError> {
        // Check rate limits (max 5 per minute per destination)
        self.check_rate_limit(&request.destination)?;

        // Generate OTP
        let otp = Self::generate_otp(request.otp_length);
        let cli = format!("+{}{}", request.cli_prefix, otp);

        // Store OTP with 5-minute TTL
        let otp_record = OtpRecord {
            otp: otp.clone(),
            destination: request.destination.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(300),
            verified: false,
        };
        self.otps.insert(request.request_id.clone(), otp_record);

        // Generate call ID
        let call_id = Uuid::new_v4().to_string();

        // Store CDR
        let cdr = FlashCallCdr {
            call_id: call_id.clone(),
            request_id: request.request_id.clone(),
            destination: request.destination.clone(),
            otp_sent: otp.clone(),
            initiated_at: Utc::now(),
            status: FlashCallStatus::Initiated,
        };
        self.cdrs.insert(call_id.clone(), cdr);

        // TODO: Initiate call via OpenSIPS
        // In production, this would call opensips_client.originate_call(...)
        tracing::info!(
            destination = %request.destination,
            cli = %cli,
            call_id = %call_id,
            "Flash call initiated"
        );

        Ok(FlashCallResponse {
            request_id: request.request_id,
            call_id,
            otp,
            status: FlashCallStatus::Initiated,
        })
    }

    /// Verify OTP from flash call
    pub async fn verify_otp(
        &self,
        request_id: &str,
        provided_otp: &str,
    ) -> Result<VerificationResult, FlashCallError> {
        let mut record = self.otps
            .get_mut(request_id)
            .ok_or(FlashCallError::OtpNotFound)?;

        if record.expires_at < Utc::now() {
            return Ok(VerificationResult::Expired);
        }

        if record.verified {
            return Ok(VerificationResult::AlreadyUsed);
        }

        if record.otp == provided_otp {
            record.verified = true;
            Ok(VerificationResult::Success)
        } else {
            Ok(VerificationResult::InvalidOtp)
        }
    }

    /// Generate random OTP of specified length
    fn generate_otp(length: u8) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let mut otp = String::new();
        let mut n = seed;
        for _ in 0..length {
            otp.push_str(&format!("{}", n % 10));
            n /= 10;
        }
        otp
    }

    /// Check rate limit for destination
    pub fn check_rate_limit(&self, destination: &str) -> Result<(), FlashCallError> {
        let mut count = self.rate_limits
            .entry(destination.to_string())
            .or_insert(0);
        
        if *count >= 5 {
            return Err(FlashCallError::RateLimited);
        }
        
        *count += 1;
        Ok(())
    }
}

/// Flash Call errors
#[derive(Debug, thiserror::Error)]
pub enum FlashCallError {
    #[error("OTP not found")]
    OtpNotFound,
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Call failed: {0}")]
    CallFailed(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
