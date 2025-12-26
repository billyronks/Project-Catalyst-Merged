//! Authentication Middleware
//!
//! JWT and API key validation.

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

pub struct AuthMiddleware {
    decoding_key: DecodingKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub roles: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(secret: String) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Validate JWT token
    pub fn validate_jwt(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        
        Ok(token_data.claims)
    }

    /// Validate API key
    pub fn validate_api_key(&self, api_key: &str) -> Result<ApiKeyInfo, AuthError> {
        // In production, validate against LumaDB
        if api_key.starts_with("brivas_") && api_key.len() > 20 {
            Ok(ApiKeyInfo {
                key_id: api_key[..15].to_string(),
                scopes: vec!["read".to_string(), "write".to_string()],
            })
        } else {
            Err(AuthError::InvalidApiKey)
        }
    }

    /// Extract token from Authorization header
    pub fn extract_token(auth_header: &str) -> Option<&str> {
        if auth_header.starts_with("Bearer ") {
            Some(&auth_header[7..])
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ApiKeyInfo {
    pub key_id: String,
    pub scopes: Vec<String>,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidToken(String),
    InvalidApiKey,
    Expired,
    MissingAuth,
}
