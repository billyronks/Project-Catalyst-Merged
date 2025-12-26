//! Auth Service
//!
//! JWT token generation and validation.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use ring::rand::SecureRandom;
use uuid::Uuid;

use crate::types::Claims;

#[derive(Clone)]
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    expiry_secs: u64,
}

impl AuthService {
    pub async fn new(secret: &str, issuer: &str, expiry_secs: u64) -> brivas_core::Result<Self> {
        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer: issuer.to_string(),
            expiry_secs,
        })
    }

    /// Generate access token
    pub fn generate_token(
        &self,
        user_id: Uuid,
        email: &str,
        roles: Vec<String>,
        tenant_id: Uuid,
    ) -> brivas_core::Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.expiry_secs as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            iss: self.issuer.clone(),
            aud: "brivas".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            email: email.to_string(),
            roles,
            tenant_id: tenant_id.to_string(),
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(token)
    }

    /// Validate token and return claims
    pub fn validate_token(&self, token: &str) -> brivas_core::Result<Claims> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Generate refresh token
    pub fn generate_refresh_token(&self) -> String {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        let mut bytes = [0u8; 32];
        ring::rand::SystemRandom::new()
            .fill(&mut bytes)
            .unwrap();
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Generate password reset token
    pub fn generate_reset_token(&self, user_id: Uuid) -> brivas_core::Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(1);

        let claims = Claims {
            sub: user_id.to_string(),
            iss: self.issuer.clone(),
            aud: "password-reset".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            email: String::new(),
            roles: vec![],
            tenant_id: String::new(),
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(token)
    }
}
