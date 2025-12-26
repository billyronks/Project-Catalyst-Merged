//! STIR/SHAKEN gRPC Client
//!
//! Client for the STIR/SHAKEN Authentication Service.

use tonic::transport::{Channel, ClientTlsConfig, Certificate, Identity};
use std::time::Duration;

/// STIR/SHAKEN client for signing and verification
#[derive(Clone)]
pub struct StirShakenClient {
    #[allow(dead_code)]
    channel: Channel,
}

impl StirShakenClient {
    /// Create a new client with mTLS
    pub async fn new(
        endpoint: &str,
        client_cert_pem: &[u8],
        client_key_pem: &[u8],
        ca_cert_pem: &[u8],
    ) -> Result<Self, ClientError> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(ca_cert_pem))
            .identity(Identity::from_pem(client_cert_pem, client_key_pem));

        let channel = Channel::from_shared(endpoint.to_string())
            .map_err(|e| ClientError::Connection(e.to_string()))?
            .tls_config(tls_config)
            .map_err(|e| ClientError::Connection(e.to_string()))?
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(10))
            .connect()
            .await
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        Ok(Self { channel })
    }

    /// Create a client without TLS (for testing)
    pub async fn new_insecure(endpoint: &str) -> Result<Self, ClientError> {
        let channel = Channel::from_shared(endpoint.to_string())
            .map_err(|e| ClientError::Connection(e.to_string()))?
            .timeout(Duration::from_secs(5))
            .connect()
            .await
            .map_err(|e| ClientError::Connection(e.to_string()))?;

        Ok(Self { channel })
    }

    /// Sign a call and return Identity header
    pub async fn sign_call(
        &self,
        orig_tn: &str,
        dest_tn: &str,
        _attestation_level: Option<crate::AttestationLevel>,
        _call_id: Option<&str>,
    ) -> Result<SignCallResult, ClientError> {
        // TODO: Call gRPC service
        // let mut client = StirShakenServiceClient::new(self.channel.clone());
        // let request = SignCallRequest { ... };
        // let response = client.sign_call(request).await?;
        
        Ok(SignCallResult {
            identity_header: format!("mock-identity-header;orig={};dest={}", orig_tn, dest_tn),
            passport: "mock-passport".to_string(),
            attestation_level: crate::AttestationLevel::C,
        })
    }

    /// Verify a call Identity header
    pub async fn verify_call(
        &self,
        identity_header: &str,
        from_tn: &str,
        to_tn: &str,
        _call_id: Option<&str>,
    ) -> Result<VerifyCallResult, ClientError> {
        // TODO: Call gRPC service
        
        Ok(VerifyCallResult {
            status: crate::VerificationStatus::Valid,
            attestation_level: crate::AttestationLevel::A,
            verified_orig_tn: from_tn.to_string(),
            verified_dest_tn: to_tn.to_string(),
            signer_spc: "1234".to_string(),
            error_detail: None,
            flags: vec![],
            identity_header: identity_header.to_string(),
        })
    }

    /// Check if we can attest for a TN
    pub async fn check_tn_authorization(
        &self,
        tn: &str,
        _customer_id: &str,
    ) -> Result<TnAuthorizationResult, ClientError> {
        // TODO: Call gRPC service
        
        Ok(TnAuthorizationResult {
            authorized: true,
            max_attestation: crate::AttestationLevel::A,
            reason: format!("TN {} is authorized", tn),
        })
    }
}

/// Result of signing a call
pub struct SignCallResult {
    pub identity_header: String,
    pub passport: String,
    pub attestation_level: crate::AttestationLevel,
}

/// Result of verifying a call
pub struct VerifyCallResult {
    pub status: crate::VerificationStatus,
    pub attestation_level: crate::AttestationLevel,
    pub verified_orig_tn: String,
    pub verified_dest_tn: String,
    pub signer_spc: String,
    pub error_detail: Option<String>,
    pub flags: Vec<String>,
    pub identity_header: String,
}

/// Result of TN authorization check
pub struct TnAuthorizationResult {
    pub authorized: bool,
    pub max_attestation: crate::AttestationLevel,
    pub reason: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("RPC error: {0}")]
    Rpc(String),
}
