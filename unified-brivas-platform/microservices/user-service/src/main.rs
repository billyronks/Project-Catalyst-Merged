//! User Service
//!
//! Comprehensive identity and access management:
//! - User identity management
//! - OAuth 2.0 / OpenID Connect
//! - Multi-factor authentication (TOTP, WebAuthn, SMS)
//! - SCIM 2.0 provisioning
//! - API key management
//! - Role-based access control

#![allow(dead_code)]

use brivas_core::{BrivasService, HealthStatus, MicroserviceRuntime, ReadinessStatus, Result};
use std::sync::Arc;
use tracing::info;

mod identity;
mod auth;
mod mfa;
mod scim;
mod api;
mod types;
mod rbac;

#[cfg(test)]
mod tests;

pub use identity::IdentityService;
pub use auth::AuthService;
pub use mfa::MfaService;
pub use scim::ScimService;
pub use rbac::RbacService;
pub use types::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("user_service=debug".parse().expect("valid tracing directive")),
        )
        .json()
        .init();

    info!("Starting User Service");

    let service = Arc::new(UserService::new().await?);
    MicroserviceRuntime::run(service).await
}

pub struct UserService {
    config: UserServiceConfig,
    identity_service: IdentityService,
    auth_service: AuthService,
    mfa_service: MfaService,
    scim_service: ScimService,
    start_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct UserServiceConfig {
    pub http_bind: String,
    pub grpc_bind: String,
    pub lumadb_url: String,
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_expiry_secs: u64,
    pub password_min_length: usize,
    pub mfa_issuer: String,
}

impl UserServiceConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_bind: std::env::var("HTTP_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            grpc_bind: std::env::var("GRPC_BIND")
                .unwrap_or_else(|_| "0.0.0.0:9090".to_string()),
            lumadb_url: std::env::var("LUMADB_URL")
                .unwrap_or_else(|_| "postgres://brivas:password@localhost:5432/brivas".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".to_string()),
            jwt_issuer: std::env::var("JWT_ISSUER")
                .unwrap_or_else(|_| "brivas".to_string()),
            jwt_expiry_secs: std::env::var("JWT_EXPIRY_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
            password_min_length: 12,
            mfa_issuer: std::env::var("MFA_ISSUER")
                .unwrap_or_else(|_| "Brivas".to_string()),
        })
    }
}

impl UserService {
    pub async fn new() -> Result<Self> {
        let config = UserServiceConfig::from_env()?;
        
        let identity_service = IdentityService::new(&config.lumadb_url).await?;
        let auth_service = AuthService::new(
            &config.jwt_secret,
            &config.jwt_issuer,
            config.jwt_expiry_secs,
        ).await?;
        let mfa_service = MfaService::new(&config.mfa_issuer).await?;
        let scim_service = ScimService::new(&config.lumadb_url).await?;

        Ok(Self {
            config,
            identity_service,
            auth_service,
            mfa_service,
            scim_service,
            start_time: std::time::Instant::now(),
        })
    }
}

#[async_trait::async_trait]
impl BrivasService for UserService {
    fn service_id(&self) -> &'static str {
        "user-service"
    }

    async fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            service_id: self.service_id().to_string(),
            version: self.version().to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    async fn ready(&self) -> ReadinessStatus {
        ReadinessStatus {
            ready: true,
            dependencies: vec![
                brivas_core::DependencyStatus {
                    name: "lumadb".to_string(),
                    available: true,
                    latency_ms: Some(1),
                },
            ],
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down User Service");
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        info!(
            http = %self.config.http_bind,
            grpc = %self.config.grpc_bind,
            "Starting User Service servers"
        );

        let rest_router = api::rest::create_router(
            self.identity_service.clone(),
            self.auth_service.clone(),
            self.mfa_service.clone(),
        );

        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        axum::serve(listener, rest_router).await?;

        Ok(())
    }
}
