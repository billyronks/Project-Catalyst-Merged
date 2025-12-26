//! Certificate Manager

use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::{Certificate, CertificateStatus, UploadCertificateRequest, Timestamp};

#[derive(Clone)]
pub struct CertificateManager {
    certificates: Arc<DashMap<String, StoredCertificate>>,
    default_cert_id: Arc<std::sync::RwLock<Option<String>>>,
    #[allow(dead_code)]
    lumadb_url: String,
}

#[derive(Clone)]
struct StoredCertificate {
    cert: Certificate,
    private_key_pem: Vec<u8>,
    public_key_hash: String,
}

impl CertificateManager {
    pub async fn new(lumadb_url: &str) -> brivas_core::Result<Self> {
        Ok(Self {
            certificates: Arc::new(DashMap::new()),
            default_cert_id: Arc::new(std::sync::RwLock::new(None)),
            lumadb_url: lumadb_url.to_string(),
        })
    }

    pub fn has_active_certificate(&self) -> bool {
        self.certificates.iter().any(|e| e.value().cert.status == CertificateStatus::Active as i32)
    }

    pub async fn get_default_certificate(&self, _pop_id: &str) -> Option<SigningCertificate> {
        let default_id = self.default_cert_id.read().ok()?.clone()?;
        self.get_signing_certificate(&default_id).await
    }

    pub async fn get_certificate(&self, id: &str) -> Option<Certificate> {
        self.certificates.get(id).map(|e| e.value().cert.clone())
    }

    pub async fn get_signing_certificate(&self, id: &str) -> Option<SigningCertificate> {
        self.certificates.get(id).map(|e| {
            let stored = e.value();
            SigningCertificate {
                id: stored.cert.id.clone(),
                certificate_url: stored.cert.certificate_url.clone(),
                private_key_pem: stored.private_key_pem.clone(),
                public_key_hash: stored.public_key_hash.clone(),
            }
        })
    }

    pub async fn list_certificates(
        &self,
        include_expired: bool,
        include_revoked: bool,
        _pop_id: &str,
    ) -> Vec<Certificate> {
        self.certificates.iter()
            .filter(|e| {
                let status = e.value().cert.status;
                if !include_expired && status == CertificateStatus::Expired as i32 { return false; }
                if !include_revoked && status == CertificateStatus::Revoked as i32 { return false; }
                true
            })
            .map(|e| e.value().cert.clone())
            .collect()
    }

    pub async fn upload_certificate(&self, req: UploadCertificateRequest) -> brivas_core::Result<Certificate> {
        let id = Uuid::new_v4().to_string();
        let cert = Certificate {
            id: id.clone(),
            name: req.name,
            subject: "CN=Brivas Telecom".to_string(),
            issuer: "CN=STI-CA".to_string(),
            spc: "1234".to_string(),
            serial_number: Uuid::new_v4().to_string(),
            not_before: None,
            not_after: None,
            public_key_algorithm: "ECDSA".to_string(),
            signature_algorithm: "SHA256withECDSA".to_string(),
            certificate_url: req.certificate_url,
            is_active: true,
            is_default: req.set_as_default,
            created_at: Some(Timestamp::from(std::time::SystemTime::now())),
            updated_at: Some(Timestamp::from(std::time::SystemTime::now())),
            status: CertificateStatus::Active as i32,
            pop_ids: req.pop_ids,
        };

        let stored = StoredCertificate {
            cert: cert.clone(),
            private_key_pem: req.private_key_pem,
            public_key_hash: "sha256-hash".to_string(),
        };

        self.certificates.insert(id.clone(), stored);

        if req.set_as_default {
            if let Ok(mut default) = self.default_cert_id.write() {
                *default = Some(id);
            }
        }

        Ok(cert)
    }

    pub async fn delete_certificate(&self, id: &str) -> brivas_core::Result<()> {
        self.certificates.remove(id);
        Ok(())
    }

    pub async fn rotate_certificate(&self, id: &str, _use_acme: bool) -> brivas_core::Result<Certificate> {
        self.get_certificate(id).await
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Certificate not found").into())
    }
}

/// Certificate ready for PASSporT signing
pub struct SigningCertificate {
    pub id: String,
    pub certificate_url: String,
    pub private_key_pem: Vec<u8>,
    pub public_key_hash: String,
}
