//! STIR/SHAKEN Types
//!
//! Local types for STIR/SHAKEN operations (replacing proto-generated types)

use serde::{Deserialize, Serialize};

/// Attestation level per ATIS-1000074
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum AttestationLevel {
    Unknown = 0,
    /// Full attestation - we are the originator
    AttestationA = 1,
    /// Partial attestation - customer authenticated
    AttestationB = 2,
    /// Gateway attestation - no authentication
    AttestationC = 3,
}

impl TryFrom<i32> for AttestationLevel {
    type Error = ();
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::AttestationA),
            2 => Ok(Self::AttestationB),
            3 => Ok(Self::AttestationC),
            _ => Err(()),
        }
    }
}

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum VerificationStatus {
    Unknown = 0,
    Valid = 1,
    InvalidSignature = 2,
    Expired = 3,
    CertRevoked = 4,
    CertNotTrusted = 5,
    TnMismatch = 6,
    NoIdentity = 7,
}

/// Certificate status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum CertificateStatus {
    Unknown = 0,
    Active = 1,
    Expired = 2,
    Revoked = 3,
    Pending = 4,
}

/// Sign call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignCallRequest {
    pub orig_tn: String,
    pub dest_tn: String,
    pub attestation_level: i32,
    pub orig_id: String,
    pub call_id: String,
    pub certificate_id: String,
    pub pop_id: String,
}

/// Sign call response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignCallResponse {
    pub identity_header: String,
    pub passport: String,
    pub attestation_level: i32,
    pub certificate_id: String,
    pub signed_at: Option<Timestamp>,
    pub expires_at: Option<Timestamp>,
}

/// Verify call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCallRequest {
    pub identity_header: String,
    pub from_tn: String,
    pub to_tn: String,
    pub call_id: String,
    pub sip_date: Option<Timestamp>,
    pub pop_id: String,
}

/// Verify call response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerifyCallResponse {
    pub status: i32,
    pub attestation_level: i32,
    pub verified_orig_tn: String,
    pub verified_dest_tn: String,
    pub orig_id: String,
    pub signer_subject: String,
    pub signer_spc: String,
    pub verified_at: Option<Timestamp>,
    pub error_detail: String,
    pub flags: Vec<String>,
}

/// Telephone number authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelephoneNumber {
    pub number: String,
    pub customer_id: String,
    pub max_attestation: i32,
    pub valid_from: Option<Timestamp>,
    pub valid_until: Option<Timestamp>,
}

/// Certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: String,
    pub name: String,
    pub subject: String,
    pub issuer: String,
    pub spc: String,
    pub serial_number: String,
    pub not_before: Option<Timestamp>,
    pub not_after: Option<Timestamp>,
    pub public_key_algorithm: String,
    pub signature_algorithm: String,
    pub certificate_url: String,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub status: i32,
    pub pop_ids: Vec<String>,
}

/// Statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsResponse {
    pub total_signs: i64,
    pub total_verifications: i64,
    pub signs_last_hour: i64,
    pub verifications_last_hour: i64,
    pub avg_sign_latency_ms: f64,
    pub avg_verify_latency_ms: f64,
    pub active_certificates: i32,
    pub registered_tns: i32,
    pub signs_by_attestation: std::collections::HashMap<String, i64>,
    pub verifications_by_status: std::collections::HashMap<String, i64>,
}

/// Simple timestamp type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
}

impl From<std::time::SystemTime> for Timestamp {
    fn from(time: std::time::SystemTime) -> Self {
        let duration = time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        Self {
            seconds: duration.as_secs() as i64,
            nanos: duration.subsec_nanos() as i32,
        }
    }
}

/// Upload certificate request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadCertificateRequest {
    pub name: String,
    pub certificate_pem: Vec<u8>,
    pub private_key_pem: Vec<u8>,
    pub certificate_url: String,
    pub set_as_default: bool,
    pub pop_ids: Vec<String>,
}
