//! Temporal Workflow SDK for Brivas Platform
//!
//! Provides abstractions and utilities for building Temporal workflows
//! and activities in the Brivas telecommunications platform.

mod error;
mod retry;
mod workflow;

pub use error::{TemporalError, Result};
pub use retry::RetryPolicy;
pub use workflow::*;

/// Re-export for convenience
pub mod prelude {
    pub use super::{TemporalError, Result, RetryPolicy};
    pub use super::workflow::*;
}
