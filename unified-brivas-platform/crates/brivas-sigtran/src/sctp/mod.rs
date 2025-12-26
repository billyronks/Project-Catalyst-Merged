//! SCTP (Stream Control Transmission Protocol) Layer
//!
//! Provides multi-streaming transport for M3UA.

mod association;
mod chunks;

pub use association::{SctpAssociation, AssociationState};

use crate::errors::SctpError;
use bytes::{Bytes, BytesMut, Buf, BufMut};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};

/// SCTP chunk types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChunkType {
    Data = 0,
    Init = 1,
    InitAck = 2,
    Sack = 3,
    Heartbeat = 4,
    HeartbeatAck = 5,
    Abort = 6,
    Shutdown = 7,
    ShutdownAck = 8,
    Error = 9,
    CookieEcho = 10,
    CookieAck = 11,
    ShutdownComplete = 14,
}

/// SCTP Payload Protocol Identifier for M3UA
pub const PPID_M3UA: u32 = 3;

/// Stream configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub inbound_streams: u16,
    pub outbound_streams: u16,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            inbound_streams: 2,
            outbound_streams: 2,
        }
    }
}
