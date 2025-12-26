//! Attestation module

mod signer;

pub use signer::AttestationSigner;
// Re-export types needed by verification
pub use signer::PassportClaims;
