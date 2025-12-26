//! Brivas LumaDB Client
//!
//! PostgreSQL wire-protocol compatible client for LumaDB.
//! Provides connection pooling, typed queries, and streaming support.

mod client;
mod pool;
mod error;
mod types;

pub use client::LumaDbClient;
pub use pool::{LumaDbPool, PoolConfig};
pub use error::{LumaDbError, Result};
pub use types::*;

/// Re-export tokio-postgres types for convenience
pub use tokio_postgres::{Row, Statement, types::ToSql};
