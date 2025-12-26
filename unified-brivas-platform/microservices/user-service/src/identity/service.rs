//! Identity Service
//!
//! User identity CRUD operations with password hashing.

use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher, PasswordVerifier, PasswordHash};
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::{User, UserStatus};

#[derive(Clone)]
pub struct IdentityService {
    users: Arc<DashMap<Uuid, User>>,
    users_by_email: Arc<DashMap<String, Uuid>>,
    #[allow(dead_code)]
    lumadb_url: String,
}

impl IdentityService {
    pub async fn new(lumadb_url: &str) -> brivas_core::Result<Self> {
        Ok(Self {
            users: Arc::new(DashMap::new()),
            users_by_email: Arc::new(DashMap::new()),
            lumadb_url: lumadb_url.to_string(),
        })
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        email: &str,
        password: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
        tenant_id: Uuid,
    ) -> brivas_core::Result<User> {
        // Check if email already exists
        if self.users_by_email.contains_key(email) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Email already registered",
            ).into());
        }

        // Hash password
        let password_hash = self.hash_password(password)?;

        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            username: None,
            password_hash,
            first_name: first_name.map(String::from),
            last_name: last_name.map(String::from),
            phone: None,
            status: UserStatus::PendingVerification,
            roles: vec!["user".to_string()],
            tenant_id,
            mfa_enabled: false,
            mfa_type: None,
            email_verified: false,
            phone_verified: false,
            last_login: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        };

        self.users.insert(user.id, user.clone());
        self.users_by_email.insert(email.to_string(), user.id);

        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(&self, id: Uuid) -> Option<User> {
        self.users.get(&id).map(|u| u.clone())
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Option<User> {
        let id = self.users_by_email.get(email)?;
        self.users.get(&*id).map(|u| u.clone())
    }

    /// Update user
    pub async fn update_user(&self, id: Uuid, updates: UserUpdate) -> brivas_core::Result<User> {
        let mut user = self.users.get_mut(&id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "User not found"))?;

        if let Some(first_name) = updates.first_name {
            user.first_name = Some(first_name);
        }
        if let Some(last_name) = updates.last_name {
            user.last_name = Some(last_name);
        }
        if let Some(phone) = updates.phone {
            user.phone = Some(phone);
        }
        if let Some(roles) = updates.roles {
            user.roles = roles;
        }
        user.updated_at = Utc::now();

        Ok(user.clone())
    }

    /// Change password
    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> brivas_core::Result<()> {
        let mut user = self.users.get_mut(&user_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "User not found"))?;

        // Verify old password
        if !self.verify_password(old_password, &user.password_hash)? {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Invalid current password",
            ).into());
        }

        user.password_hash = self.hash_password(new_password)?;
        user.updated_at = Utc::now();

        Ok(())
    }

    /// Verify password
    pub fn verify_password(&self, password: &str, hash: &str) -> brivas_core::Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Hash password
    fn hash_password(&self, password: &str) -> brivas_core::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
            .to_string();
        
        Ok(hash)
    }

    /// Verify email
    pub async fn verify_email(&self, user_id: Uuid) -> brivas_core::Result<()> {
        if let Some(mut user) = self.users.get_mut(&user_id) {
            user.email_verified = true;
            user.status = UserStatus::Active;
            user.updated_at = Utc::now();
        }
        Ok(())
    }

    /// Record login
    pub async fn record_login(&self, user_id: Uuid) -> brivas_core::Result<()> {
        if let Some(mut user) = self.users.get_mut(&user_id) {
            user.last_login = Some(Utc::now());
        }
        Ok(())
    }

    /// List users with pagination
    pub async fn list_users(&self, tenant_id: Uuid, limit: usize, offset: usize) -> Vec<User> {
        self.users
            .iter()
            .filter(|u| u.value().tenant_id == tenant_id)
            .skip(offset)
            .take(limit)
            .map(|u| u.value().clone())
            .collect()
    }
}

pub struct UserUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub roles: Option<Vec<String>>,
}
