//! MFA Service
//!
//! Multi-factor authentication with TOTP.

use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::types::{MfaSecret, MfaType};

#[derive(Clone)]
pub struct MfaService {
    secrets: Arc<DashMap<Uuid, MfaSecret>>,
    issuer: String,
}

impl MfaService {
    pub async fn new(issuer: &str) -> brivas_core::Result<Self> {
        Ok(Self {
            secrets: Arc::new(DashMap::new()),
            issuer: issuer.to_string(),
        })
    }

    /// Enable TOTP for user
    pub async fn enable_totp(&self, user_id: Uuid, email: &str) -> brivas_core::Result<TotpSetup> {
        // Generate secret
        let secret = Secret::generate_secret();
        
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().unwrap(),
            Some(self.issuer.clone()),
            email.to_string(),
        ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        // Generate backup codes
        let backup_codes: Vec<String> = (0..10)
            .map(|_| format!("{:08}", rand::random::<u32>() % 100_000_000))
            .collect();

        let mfa_secret = MfaSecret {
            user_id,
            mfa_type: MfaType::Totp,
            secret: secret.to_encoded().to_string(),
            backup_codes: backup_codes.clone(),
            verified: false,
            created_at: Utc::now(),
        };

        self.secrets.insert(user_id, mfa_secret);

        // Generate QR code URL
        let qr_url = totp.get_url();

        Ok(TotpSetup {
            secret: secret.to_encoded().to_string(),
            qr_url,
            backup_codes,
        })
    }

    /// Verify TOTP code
    pub async fn verify_totp(&self, user_id: Uuid, code: &str) -> brivas_core::Result<bool> {
        let mfa_secret = self.secrets.get(&user_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "MFA not configured"))?;

        let secret = Secret::Encoded(mfa_secret.secret.clone())
            .to_bytes()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret, Some(self.issuer.clone()), "user".to_string())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(totp.check_current(code).unwrap_or(false))
    }

    /// Confirm TOTP setup with verification code
    pub async fn confirm_totp(&self, user_id: Uuid, code: &str) -> brivas_core::Result<bool> {
        if self.verify_totp(user_id, code).await? {
            if let Some(mut secret) = self.secrets.get_mut(&user_id) {
                secret.verified = true;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check backup code
    pub async fn use_backup_code(&self, user_id: Uuid, code: &str) -> brivas_core::Result<bool> {
        if let Some(mut secret) = self.secrets.get_mut(&user_id) {
            if let Some(pos) = secret.backup_codes.iter().position(|c| c == code) {
                secret.backup_codes.remove(pos);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Disable MFA
    pub async fn disable_mfa(&self, user_id: Uuid) -> brivas_core::Result<()> {
        self.secrets.remove(&user_id);
        Ok(())
    }

    /// Check if MFA is enabled
    pub fn is_enabled(&self, user_id: Uuid) -> bool {
        self.secrets.get(&user_id)
            .map(|s| s.verified)
            .unwrap_or(false)
    }
}

pub struct TotpSetup {
    pub secret: String,
    pub qr_url: String,
    pub backup_codes: Vec<String>,
}
