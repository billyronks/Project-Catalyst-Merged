//! SCIM 2.0 Service
//!
//! System for Cross-domain Identity Management provisioning.

use crate::identity::IdentityService;
use crate::types::{ScimEmail, ScimMeta, ScimName, ScimUser, User};
use rand::Rng;
use uuid::Uuid;

#[derive(Clone)]
pub struct ScimService {
    #[allow(dead_code)]
    lumadb_url: String,
}

impl ScimService {
    pub async fn new(lumadb_url: &str) -> brivas_core::Result<Self> {
        Ok(Self {
            lumadb_url: lumadb_url.to_string(),
        })
    }

    /// Convert User to SCIM representation
    pub fn to_scim_user(&self, user: &User) -> ScimUser {
        ScimUser {
            schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:User".to_string()],
            id: user.id.to_string(),
            user_name: user.email.clone(),
            name: ScimName {
                given_name: user.first_name.clone(),
                family_name: user.last_name.clone(),
            },
            emails: vec![ScimEmail {
                value: user.email.clone(),
                primary: true,
            }],
            active: user.status == crate::types::UserStatus::Active,
            meta: ScimMeta {
                resource_type: "User".to_string(),
                created: user.created_at.to_rfc3339(),
                last_modified: user.updated_at.to_rfc3339(),
            },
        }
    }

    /// Create user from SCIM request
    pub async fn create_from_scim(
        &self,
        scim_user: &ScimUser,
        identity_service: &IdentityService,
        tenant_id: Uuid,
    ) -> brivas_core::Result<User> {
        let email = scim_user.emails.first()
            .map(|e| e.value.as_str())
            .unwrap_or(&scim_user.user_name);

        // Generate temporary password
        let temp_password = format!("TempPass{}!", rand::thread_rng().gen::<u32>());

        identity_service.create_user(
            email,
            &temp_password,
            scim_user.name.given_name.as_deref(),
            scim_user.name.family_name.as_deref(),
            tenant_id,
        ).await
    }
}
