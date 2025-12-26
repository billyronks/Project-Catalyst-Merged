//! PASSporT Verification Service

use chrono::Utc;
use dashmap::DashMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use std::sync::Arc;

use crate::attestation::PassportClaims;
use crate::config::StirShakenConfig;
use crate::types::{AttestationLevel, VerificationStatus, VerifyCallRequest, VerifyCallResponse, Timestamp};

#[derive(Clone)]
pub struct VerificationService {
    http_client: Client,
    cert_cache: Arc<DashMap<String, CachedCertificate>>,
    #[allow(dead_code)]
    sti_ca_urls: Vec<String>,
}

#[derive(Clone)]
struct CachedCertificate {
    public_key_pem: Vec<u8>,
    subject: String,
    spc: String,
    expires_at: chrono::DateTime<Utc>,
}

impl VerificationService {
    pub async fn new(config: &StirShakenConfig) -> brivas_core::Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(Self {
            http_client,
            cert_cache: Arc::new(DashMap::new()),
            sti_ca_urls: config.sti_ca_urls.clone(),
        })
    }

    pub async fn verify(&self, request: &VerifyCallRequest) -> Result<VerifyCallResponse, VerifyError> {
        if request.identity_header.is_empty() {
            return Ok(VerifyCallResponse {
                status: VerificationStatus::NoIdentity as i32,
                error_detail: "No Identity header present".to_string(),
                verified_at: Some(Timestamp::from(std::time::SystemTime::now())),
                ..Default::default()
            });
        }

        let (passport, cert_url) = self.parse_identity_header(&request.identity_header)?;
        let signing_cert = self.fetch_certificate(&cert_url).await?;

        let mut validation = Validation::new(Algorithm::ES256);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();

        let decoding_key = DecodingKey::from_ec_pem(&signing_cert.public_key_pem)
            .map_err(|e| VerifyError::InvalidCertificate(e.to_string()))?;

        let token_data = match decode::<PassportClaims>(&passport, &decoding_key, &validation) {
            Ok(data) => data,
            Err(e) => {
                return Ok(VerifyCallResponse {
                    status: VerificationStatus::InvalidSignature as i32,
                    error_detail: format!("Invalid signature: {}", e),
                    verified_at: Some(Timestamp::from(std::time::SystemTime::now())),
                    ..Default::default()
                });
            }
        };

        let claims = token_data.claims;

        let iat = chrono::DateTime::from_timestamp(claims.iat, 0)
            .ok_or(VerifyError::InvalidTimestamp)?;
        let age = Utc::now() - iat;
        if age.num_seconds() > 60 || age.num_seconds() < -5 {
            return Ok(VerifyCallResponse {
                status: VerificationStatus::Expired as i32,
                error_detail: format!("PASSporT age {} seconds exceeds limit", age.num_seconds()),
                verified_at: Some(Timestamp::from(std::time::SystemTime::now())),
                ..Default::default()
            });
        }

        let normalized_from = normalize_tn(&request.from_tn);
        if claims.orig.tn != normalized_from {
            return Ok(VerifyCallResponse {
                status: VerificationStatus::TnMismatch as i32,
                error_detail: format!("TN mismatch: {} vs {}", claims.orig.tn, normalized_from),
                verified_at: Some(Timestamp::from(std::time::SystemTime::now())),
                ..Default::default()
            });
        }

        let attestation = string_to_attestation(&claims.attest);

        Ok(VerifyCallResponse {
            status: VerificationStatus::Valid as i32,
            attestation_level: attestation as i32,
            verified_orig_tn: claims.orig.tn,
            verified_dest_tn: claims.dest.tn.first().cloned().unwrap_or_default(),
            orig_id: claims.origid.unwrap_or_default(),
            signer_subject: signing_cert.subject,
            signer_spc: signing_cert.spc,
            verified_at: Some(Timestamp::from(std::time::SystemTime::now())),
            error_detail: String::new(),
            flags: vec![],
        })
    }

    fn parse_identity_header(&self, header: &str) -> Result<(String, String), VerifyError> {
        let parts: Vec<&str> = header.split(';').collect();
        if parts.is_empty() {
            return Err(VerifyError::InvalidHeader("Empty".to_string()));
        }
        let passport = parts[0].to_string();
        let mut cert_url = String::new();
        for part in &parts[1..] {
            let part = part.trim();
            if part.starts_with("info=<") && part.ends_with('>') {
                cert_url = part[6..part.len() - 1].to_string();
                break;
            }
        }
        if cert_url.is_empty() {
            return Err(VerifyError::InvalidHeader("Missing info".to_string()));
        }
        Ok((passport, cert_url))
    }

    async fn fetch_certificate(&self, url: &str) -> Result<CachedCertificate, VerifyError> {
        if let Some(cached) = self.cert_cache.get(url) {
            if cached.expires_at > Utc::now() {
                return Ok(cached.clone());
            }
        }

        let response = self.http_client
            .get(url)
            .header("Accept", "application/x-x509-ca-cert")
            .send()
            .await
            .map_err(|e| VerifyError::FetchFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(VerifyError::FetchFailed(format!("HTTP {}", response.status())));
        }

        let cert_pem = response.bytes().await
            .map_err(|e| VerifyError::FetchFailed(e.to_string()))?;

        let cached = CachedCertificate {
            public_key_pem: cert_pem.to_vec(),
            subject: "CN=Unknown".to_string(),
            spc: String::new(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        self.cert_cache.insert(url.to_string(), cached.clone());
        Ok(cached)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("Invalid header: {0}")]
    InvalidHeader(String),
    #[error("Fetch failed: {0}")]
    FetchFailed(String),
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

fn string_to_attestation(s: &str) -> AttestationLevel {
    match s.to_uppercase().as_str() {
        "A" => AttestationLevel::AttestationA,
        "B" => AttestationLevel::AttestationB,
        "C" => AttestationLevel::AttestationC,
        _ => AttestationLevel::Unknown,
    }
}

fn normalize_tn(tn: &str) -> String {
    let has_plus = tn.starts_with('+');
    let digits: String = tn.chars().filter(|c| c.is_ascii_digit()).collect();
    if has_plus { format!("+{}", digits) } else { digits }
}
