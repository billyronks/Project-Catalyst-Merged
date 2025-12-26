//! Attestation types

use serde::{Deserialize, Serialize};

/// Attestation level per ATIS-1000074
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationLevel {
    /// Full attestation - we are the originator
    A,
    /// Partial attestation - customer authenticated
    B,
    /// Gateway attestation - no authentication
    C,
    /// Unknown attestation level
    Unknown,
}

impl AttestationLevel {
    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "A" => Self::A,
            "B" => Self::B,
            "C" => Self::C,
            _ => Self::Unknown,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::Unknown => "?",
        }
    }
}

impl std::fmt::Display for AttestationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// PASSporT is valid
    Valid,
    /// Invalid signature
    InvalidSignature,
    /// PASSporT expired
    Expired,
    /// Certificate revoked
    CertRevoked,
    /// Certificate not trusted
    CertNotTrusted,
    /// TN mismatch between PASSporT and SIP headers
    TnMismatch,
    /// No Identity header present
    NoIdentity,
    /// Unknown error
    Unknown,
}

impl VerificationStatus {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Valid => "VALID",
            Self::InvalidSignature => "INVALID_SIGNATURE",
            Self::Expired => "EXPIRED",
            Self::CertRevoked => "CERT_REVOKED",
            Self::CertNotTrusted => "CERT_NOT_TRUSTED",
            Self::TnMismatch => "TN_MISMATCH",
            Self::NoIdentity => "NO_IDENTITY",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
