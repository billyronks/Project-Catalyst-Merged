//! kdb+ IPC SDK for Brivas Platform
//!
//! Provides a Rust client for communicating with kdb+ databases
//! using the kdb+ IPC protocol.

mod client;
mod error;
mod types;

pub use client::KdbClient;
pub use error::{KdbError, Result};
pub use types::*;

/// Re-export for convenience
pub mod prelude {
    pub use super::{KdbClient, KdbError, Result};
    pub use super::types::*;
}
