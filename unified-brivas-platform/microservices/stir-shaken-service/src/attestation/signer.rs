//! PASSporT Attestation Signer
//!
//! Signs outbound calls with STIR/SHAKEN PASSporT tokens.

use chrono::{Duration, Utc};
use dashmap::DashMap;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::certificate::CertificateManager;
use crate::types::{
    AttestationLevel, SignCallRequest, SignCallResponse, StatisticsResponse, TelephoneNumber,
    Timestamp,
};

/// PASSporT Claims per RFC 8225
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportClaims {
    pub attest: String,
    pub dest: DestinationIdentity,
    pub iat: i64,
    pub orig: OriginatingIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationIdentity {
    pub tn: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginatingIdentity {
    pub tn: String,
}

#[derive(Clone)]
pub struct AttestationSigner {
    cert_manager: CertificateManager,
    tn_cache: Arc<DashMap<String, TnAuthorization>>,
    stats: Arc<SigningStats>,
}

#[derive(Clone)]
struct TnAuthorization {
    customer_id: String,
    max_attestation: AttestationLevel,
    valid_until: chrono::DateTime<Utc>,
}

struct SigningStats {
    total_signs: std::sync::atomic::AtomicU64,
    signs_a: std::sync::atomic::AtomicU64,
    signs_b: std::sync::atomic::AtomicU64,
    signs_c: std::sync::atomic::AtomicU64,
}

impl Default for SigningStats {
    fn default() -> Self {
        Self {
            total_signs: std::sync::atomic::AtomicU64::new(0),
            signs_a: std::sync::atomic::AtomicU64::new(0),
            signs_b: std::sync::atomic::AtomicU64::new(0),
            signs_c: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl AttestationSigner {
    pub fn new(cert_manager: CertificateManager) -> Self {
        Self {
            cert_manager,
            tn_cache: Arc::new(DashMap::new()),
            stats: Arc::new(SigningStats::default()),
        }
    }

    pub async fn sign(&self, request: &SignCallRequest) -> Result<SignCallResponse, SignError> {
        let attestation = if request.attestation_level != 0 {
            AttestationLevel::try_from(request.attestation_level)
                .unwrap_or(AttestationLevel::AttestationC)
        } else {
            let (level, _) = self.determine_attestation_level(&request.orig_tn, "").await;
            level
        };

        let cert = if request.certificate_id.is_empty() {
            self.cert_manager
                .get_default_certificate(&request.pop_id)
                .await
                .ok_or(SignError::NoCertificate)?
        } else {
            self.cert_manager
                .get_signing_certificate(&request.certificate_id)
                .await
                .ok_or(SignError::CertificateNotFound)?
        };

        let now = Utc::now();
        let claims = PassportClaims {
            attest: attestation_to_string(attestation),
            dest: DestinationIdentity {
                tn: vec![normalize_tn(&request.dest_tn)],
            },
            iat: now.timestamp(),
            orig: OriginatingIdentity {
                tn: normalize_tn(&request.orig_tn),
            },
            origid: if request.orig_id.is_empty() {
                Some(Uuid::new_v4().to_string())
            } else {
                Some(request.orig_id.clone())
            },
        };

        let mut header = Header::new(Algorithm::ES256);
        header.typ = Some("passport".to_string());
        header.kid = Some(cert.public_key_hash.clone());

        let encoding_key = EncodingKey::from_ec_pem(&cert.private_key_pem)
            .map_err(|e| SignError::SigningFailed(e.to_string()))?;

        let passport = encode(&header, &claims, &encoding_key)
            .map_err(|e| SignError::SigningFailed(e.to_string()))?;

        let identity_header = format!(
            "{};info=<{}>;alg=ES256;ppt=shaken",
            passport, cert.certificate_url
        );

        self.stats.total_signs.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match attestation {
            AttestationLevel::AttestationA => {
                self.stats.signs_a.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            AttestationLevel::AttestationB => {
                self.stats.signs_b.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            _ => {
                self.stats.signs_c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        Ok(SignCallResponse {
            identity_header,
            passport,
            attestation_level: attestation as i32,
            certificate_id: cert.id,
            signed_at: Some(Timestamp::from(std::time::SystemTime::now())),
            expires_at: Some(Timestamp::from(
                std::time::SystemTime::now() + std::time::Duration::from_secs(60),
            )),
        })
    }

    pub async fn determine_attestation_level(
        &self,
        tn: &str,
        customer_id: &str,
    ) -> (AttestationLevel, bool) {
        let normalized = normalize_tn(tn);
        if let Some(auth) = self.tn_cache.get(&normalized) {
            if auth.valid_until > Utc::now() {
                if customer_id.is_empty() || auth.customer_id == customer_id {
                    return (auth.max_attestation, true);
                }
            }
        }
        (AttestationLevel::AttestationC, false)
    }

    pub async fn register_tns(
        &self,
        numbers: Vec<TelephoneNumber>,
    ) -> Result<(i32, i32, Vec<String>), SignError> {
        let mut registered = 0i32;
        let failed = 0i32;
        let errors = Vec::new();

        for tn in numbers {
            let normalized = normalize_tn(&tn.number);
            let attestation = AttestationLevel::try_from(tn.max_attestation)
                .unwrap_or(AttestationLevel::AttestationC);

            let valid_until = tn.valid_until
                .and_then(|t| chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32))
                .unwrap_or_else(|| Utc::now() + Duration::days(365));

            self.tn_cache.insert(
                normalized,
                TnAuthorization {
                    customer_id: tn.customer_id,
                    max_attestation: attestation,
                    valid_until,
                },
            );
            registered += 1;
        }

        Ok((registered, failed, errors))
    }

    pub async fn get_statistics(&self) -> StatisticsResponse {
        StatisticsResponse {
            total_signs: self.stats.total_signs.load(std::sync::atomic::Ordering::Relaxed) as i64,
            total_verifications: 0,
            signs_last_hour: 0,
            verifications_last_hour: 0,
            avg_sign_latency_ms: 2.0,
            avg_verify_latency_ms: 5.0,
            active_certificates: 1,
            registered_tns: self.tn_cache.len() as i32,
            signs_by_attestation: std::collections::HashMap::from([
                ("A".to_string(), self.stats.signs_a.load(std::sync::atomic::Ordering::Relaxed) as i64),
                ("B".to_string(), self.stats.signs_b.load(std::sync::atomic::Ordering::Relaxed) as i64),
                ("C".to_string(), self.stats.signs_c.load(std::sync::atomic::Ordering::Relaxed) as i64),
            ]),
            verifications_by_status: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SignError {
    #[error("No active certificate available")]
    NoCertificate,
    #[error("Certificate not found")]
    CertificateNotFound,
    #[error("Signing failed: {0}")]
    SigningFailed(String),
}

fn attestation_to_string(level: AttestationLevel) -> String {
    match level {
        AttestationLevel::AttestationA => "A".to_string(),
        AttestationLevel::AttestationB => "B".to_string(),
        AttestationLevel::AttestationC => "C".to_string(),
        _ => "C".to_string(),
    }
}

fn normalize_tn(tn: &str) -> String {
    let has_plus = tn.starts_with('+');
    let digits: String = tn.chars().filter(|c| c.is_ascii_digit()).collect();
    if has_plus { format!("+{}", digits) } else { digits }
}
