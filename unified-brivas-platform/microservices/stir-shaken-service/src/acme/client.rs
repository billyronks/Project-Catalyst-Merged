//! ACME Client for STI-PA Integration
//!
//! Automated certificate issuance using ACME protocol with STI-PA.

use reqwest::Client;

/// ACME client for STIR/SHAKEN certificate issuance
pub struct AcmeClient {
    http_client: Client,
    sti_pa_url: String,
    #[allow(dead_code)]
    account_url: Option<String>,
}

impl AcmeClient {
    pub fn new(sti_pa_url: &str) -> Self {
        Self {
            http_client: Client::new(),
            sti_pa_url: sti_pa_url.to_string(),
            account_url: None,
        }
    }

    /// Register with STI-PA
    pub async fn register(&mut self, _contact_email: &str) -> Result<String, AcmeError> {
        // TODO: Implement ACME account registration
        // POST to {sti_pa_url}/acme/new-acct
        Ok("account-url".to_string())
    }

    /// Request a new certificate
    pub async fn request_certificate(
        &self,
        _spc: &str,
        _csr_pem: &[u8],
    ) -> Result<IssuedCertificate, AcmeError> {
        // TODO: Implement certificate request
        // 1. POST to {sti_pa_url}/acme/new-order with TNAuthList
        // 2. Complete any required challenges
        // 3. Finalize order with CSR
        // 4. Download certificate
        
        Err(AcmeError::NotImplemented)
    }

    /// Renew an existing certificate
    pub async fn renew_certificate(
        &self,
        _current_cert_url: &str,
    ) -> Result<IssuedCertificate, AcmeError> {
        // TODO: Implement certificate renewal
        Err(AcmeError::NotImplemented)
    }

    /// Revoke a certificate
    pub async fn revoke_certificate(&self, _cert_pem: &[u8]) -> Result<(), AcmeError> {
        // TODO: Implement certificate revocation
        Ok(())
    }

    /// Get directory URLs from STI-PA
    pub async fn get_directory(&self) -> Result<AcmeDirectory, AcmeError> {
        let url = format!("{}/acme/directory", self.sti_pa_url);
        let response = self.http_client.get(&url).send().await?;
        let directory: AcmeDirectory = response.json().await?;
        Ok(directory)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AcmeDirectory {
    pub new_nonce: String,
    pub new_account: String,
    pub new_order: String,
    pub revoke_cert: String,
}

pub struct IssuedCertificate {
    pub certificate_pem: Vec<u8>,
    pub certificate_url: String,
    pub not_before: chrono::DateTime<chrono::Utc>,
    pub not_after: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum AcmeError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Challenge failed")]
    ChallengeFailed,
    #[error("Not implemented")]
    NotImplemented,
}
