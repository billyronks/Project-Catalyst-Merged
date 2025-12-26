//! BRIVAS STIR/SHAKEN SDK
//!
//! gRPC client and types for STIR/SHAKEN authentication service.

pub mod client;
pub mod passport;
pub mod attestation;

pub use client::StirShakenClient;
pub use attestation::{AttestationLevel, VerificationStatus};
