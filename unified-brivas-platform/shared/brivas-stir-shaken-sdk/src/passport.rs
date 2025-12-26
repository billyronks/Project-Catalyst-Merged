//! PASSporT types

use serde::{Deserialize, Serialize};

/// PASSporT Claims per RFC 8225
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportClaims {
    /// Attestation level: A, B, or C
    pub attest: String,
    /// Destination identity
    pub dest: DestinationIdentity,
    /// Issued at timestamp
    pub iat: i64,
    /// Originating identity
    pub orig: OriginatingIdentity,
    /// Origination identifier
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

/// Parsed Identity header
#[derive(Debug, Clone)]
pub struct IdentityHeader {
    pub passport: String,
    pub info_url: String,
    pub algorithm: String,
    pub ppt: String,
}

impl IdentityHeader {
    /// Parse an Identity header string
    pub fn parse(header: &str) -> Result<Self, PassportError> {
        let parts: Vec<&str> = header.split(';').collect();
        
        if parts.is_empty() {
            return Err(PassportError::InvalidFormat("Empty header".to_string()));
        }

        let passport = parts[0].to_string();
        let mut info_url = String::new();
        let mut algorithm = "ES256".to_string();
        let mut ppt = "shaken".to_string();

        for part in &parts[1..] {
            let part = part.trim();
            if part.starts_with("info=<") && part.ends_with('>') {
                info_url = part[6..part.len() - 1].to_string();
            } else if part.starts_with("alg=") {
                algorithm = part[4..].to_string();
            } else if part.starts_with("ppt=") {
                ppt = part[4..].to_string();
            }
        }

        if info_url.is_empty() {
            return Err(PassportError::InvalidFormat("Missing info parameter".to_string()));
        }

        Ok(Self {
            passport,
            info_url,
            algorithm,
            ppt,
        })
    }

    /// Format as Identity header string
    pub fn to_header_string(&self) -> String {
        format!(
            "{};info=<{}>;alg={};ppt={}",
            self.passport, self.info_url, self.algorithm, self.ppt
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PassportError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Expired")]
    Expired,
}
