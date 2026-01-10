//! Brivas Core - Shared domain types and service infrastructure
//!
//! This crate provides:
//! - Standard service trait all microservices must implement
//! - Common domain types (MessageId, AccountId, etc.)
//! - Error handling utilities
//! - Configuration management
//! - Service discovery for detachable microservices

pub mod config;
pub mod discovery;
pub mod domain;
pub mod error;
pub mod service;

pub use config::ServiceConfig;
pub use discovery::{DiscoveryMode, ServiceDiscovery, ServiceEndpoint};
pub use domain::*;
pub use error::{BrivasError, Result};
pub use service::{BrivasService, DependencyStatus, HealthStatus, MicroserviceRuntime, ReadinessStatus};

