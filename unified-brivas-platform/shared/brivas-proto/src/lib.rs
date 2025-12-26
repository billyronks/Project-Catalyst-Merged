//! Brivas Protocol Buffers
//!
//! gRPC service definitions and message types for all Brivas microservices.

// Include generated code (when proto files are compiled)
// tonic::include_proto!("brivas.voice");
// tonic::include_proto!("brivas.messaging");
// tonic::include_proto!("brivas.common");

pub mod common;
pub mod voice;
pub mod messaging;
pub mod smsc;

pub use common::*;
