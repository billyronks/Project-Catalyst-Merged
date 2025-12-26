//! # Brivas SIGTRAN Protocol Stack
//!
//! Production-ready implementation of SS7 over IP protocols:
//!
//! - **SCTP** - Stream Control Transmission Protocol
//! - **M3UA** - MTP3 User Adaptation Layer
//! - **SCCP** - Signaling Connection Control Part
//! - **TCAP** - Transaction Capabilities Application Part
//! - **MAP** - Mobile Application Part (SMS & USSD)
//!
//! ## Performance Target
//! 100,000+ transactions per second per PoP
//!
//! ## Example
//! ```rust,ignore
//! use brivas_sigtran::{SigtranConfig, MapEndpoint};
//!
//! let config = SigtranConfig::default();
//! let mut map = MapEndpoint::new(config).await?;
//!
//! // Send MO SMS
//! map.mo_forward_sm("+1234567890", "+0987654321", b"Hello").await?;
//! ```

pub mod sctp;
pub mod m3ua;
pub mod sccp;
pub mod tcap;
pub mod map;
pub mod types;
pub mod errors;
pub mod config;

// Re-exports
pub use config::SigtranConfig;
pub use errors::{SigtranError, Result};
pub use types::*;

// Protocol layer exports
pub use sctp::SctpAssociation;
pub use m3ua::{M3uaEndpoint, AspState};
pub use sccp::{SccpEndpoint, SccpAddress, GlobalTitle};
pub use tcap::{TcapEndpoint, TcapMessage, Component};
pub use map::{MapEndpoint, MapSmsOperation, MapUssdOperation};

/// Protocol version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default SCTP port for M3UA
pub const DEFAULT_M3UA_PORT: u16 = 2905;

/// Service Indicator for SCCP
pub const SI_SCCP: u8 = 3;

/// Subsystem Numbers
pub mod ssn {
    pub const HLR: u8 = 6;
    pub const VLR: u8 = 7;
    pub const MSC: u8 = 8;
    pub const SMSC: u8 = 8;
    pub const GSMSCF: u8 = 147;
    pub const USSD: u8 = 147;
}
