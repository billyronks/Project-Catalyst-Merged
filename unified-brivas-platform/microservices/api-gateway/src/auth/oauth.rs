//! OAuth2/OIDC Provider Integration
//!
//! Supports: Google, Microsoft, GitHub SSO

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OAuthProviderRegistry {
    providers: HashMap<String, OAuthProvider>,
}

#[derive(Debug, Clone)]
pub struct OAuthProvider {
    pub provider_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub provider: String,
}

impl OAuthProviderRegistry {
    pub fn new() -> Self {
        let mut providers = HashMap::new();
        
        // Google OAuth2
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("GOOGLE_CLIENT_ID"),
            std::env::var("GOOGLE_CLIENT_SECRET")
        ) {
            providers.insert("google".to_string(), OAuthProvider {
                provider_id: "google".to_string(),
                client_id,
                client_secret,
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                userinfo_url: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
                scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
                redirect_uri: std::env::var("OAUTH_REDIRECT_URI")
                    .unwrap_or_else(|_| "https://api.brivas.io/auth/callback/google".to_string()),
            });
        }
        
        // Microsoft OAuth2
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("MICROSOFT_CLIENT_ID"),
            std::env::var("MICROSOFT_CLIENT_SECRET")
        ) {
            let tenant = std::env::var("MICROSOFT_TENANT_ID").unwrap_or_else(|_| "common".to_string());
            providers.insert("microsoft".to_string(), OAuthProvider {
                provider_id: "microsoft".to_string(),
                client_id,
                client_secret,
                auth_url: format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize", tenant),
                token_url: format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant),
                userinfo_url: "https://graph.microsoft.com/v1.0/me".to_string(),
                scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
                redirect_uri: std::env::var("OAUTH_REDIRECT_URI")
                    .unwrap_or_else(|_| "https://api.brivas.io/auth/callback/microsoft".to_string()),
            });
        }
        
        // GitHub OAuth2
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("GITHUB_CLIENT_ID"),
            std::env::var("GITHUB_CLIENT_SECRET")
        ) {
            providers.insert("github".to_string(), OAuthProvider {
                provider_id: "github".to_string(),
                client_id,
                client_secret,
                auth_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                userinfo_url: "https://api.github.com/user".to_string(),
                scopes: vec!["user:email".to_string()],
                redirect_uri: std::env::var("OAUTH_REDIRECT_URI")
                    .unwrap_or_else(|_| "https://api.brivas.io/auth/callback/github".to_string()),
            });
        }
        
        Self { providers }
    }
    
    pub fn get(&self, provider_id: &str) -> Option<&OAuthProvider> {
        self.providers.get(provider_id)
    }
    
    pub fn list(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

impl OAuthProvider {
    /// Generate authorization URL with state parameter
    pub fn authorization_url(&self, state: &str) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            self.auth_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state)
        )
    }
    
    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse, OAuthError> {
        let client = reqwest::Client::new();
        
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ];
        
        let response = client
            .post(&self.token_url)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| OAuthError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::TokenExchange(error_text));
        }
        
        response.json().await.map_err(|e| OAuthError::Parse(e.to_string()))
    }
    
    /// Fetch user info using access token
    pub async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo, OAuthError> {
        let client = reqwest::Client::new();
        
        let response = client
            .get(&self.userinfo_url)
            .bearer_auth(access_token)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| OAuthError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(OAuthError::UserInfo("Failed to fetch user info".to_string()));
        }
        
        let data: serde_json::Value = response.json().await
            .map_err(|e| OAuthError::Parse(e.to_string()))?;
        
        // Normalize user info across providers
        let user_info = match self.provider_id.as_str() {
            "google" => OAuthUserInfo {
                id: data["sub"].as_str().unwrap_or_default().to_string(),
                email: data["email"].as_str().map(String::from),
                name: data["name"].as_str().map(String::from),
                picture: data["picture"].as_str().map(String::from),
                provider: "google".to_string(),
            },
            "microsoft" => OAuthUserInfo {
                id: data["id"].as_str().unwrap_or_default().to_string(),
                email: data["mail"].as_str().or(data["userPrincipalName"].as_str()).map(String::from),
                name: data["displayName"].as_str().map(String::from),
                picture: None,
                provider: "microsoft".to_string(),
            },
            "github" => OAuthUserInfo {
                id: data["id"].as_i64().map(|i| i.to_string()).unwrap_or_default(),
                email: data["email"].as_str().map(String::from),
                name: data["name"].as_str().map(String::from),
                picture: data["avatar_url"].as_str().map(String::from),
                provider: "github".to_string(),
            },
            _ => return Err(OAuthError::UnsupportedProvider(self.provider_id.clone())),
        };
        
        Ok(user_info)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Token exchange failed: {0}")]
    TokenExchange(String),
    #[error("Failed to fetch user info: {0}")]
    UserInfo(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
    #[error("Invalid state")]
    InvalidState,
}

impl Default for OAuthProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
