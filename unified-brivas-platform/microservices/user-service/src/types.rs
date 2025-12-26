//! User Service Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub status: UserStatus,
    pub roles: Vec<String>,
    pub tenant_id: Uuid,
    pub mfa_enabled: bool,
    pub mfa_type: Option<MfaType>,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaType {
    Totp,
    Sms,
    WebAuthn,
    Email,
}

/// API Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// OAuth Session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub email: String,
    pub roles: Vec<String>,
    pub tenant_id: String,
}

/// MFA Secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSecret {
    pub user_id: Uuid,
    pub mfa_type: MfaType,
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

/// SCIM User representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUser {
    pub schemas: Vec<String>,
    pub id: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    pub name: ScimName,
    pub emails: Vec<ScimEmail>,
    pub active: bool,
    pub meta: ScimMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimName {
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    #[serde(rename = "familyName")]
    pub family_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimEmail {
    pub value: String,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMeta {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub created: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
}
