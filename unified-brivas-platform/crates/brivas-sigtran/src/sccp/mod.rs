//! SCCP (Signaling Connection Control Part)
//!
//! ITU-T Q.711-Q.716 compliant implementation.

mod messages;
mod address;
mod gtt;

pub use messages::SccpMessage;
pub use address::{SccpAddress, GlobalTitle};
pub use gtt::GlobalTitleTranslator;

use crate::errors::SccpError;
use crate::m3ua::M3uaEndpoint;
use crate::types::{ProtocolClass, Segmentation};
use bytes::{Bytes, BytesMut, Buf, BufMut};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// SCCP Message Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Cr = 0x01,       // Connection Request
    Cc = 0x02,       // Connection Confirm
    Cref = 0x03,     // Connection Refused
    Rlsd = 0x04,     // Released
    Rlc = 0x05,      // Release Complete
    Dt1 = 0x06,      // Data Form 1
    Dt2 = 0x07,      // Data Form 2
    Ak = 0x08,       // Data Acknowledgement
    Udt = 0x09,      // Unitdata
    Udts = 0x0A,     // Unitdata Service
    Ed = 0x0B,       // Expedited Data
    Ea = 0x0C,       // Expedited Data Acknowledgement
    Rsr = 0x0D,      // Reset Request
    Rsc = 0x0E,      // Reset Confirm
    Err = 0x0F,      // Protocol Error
    It = 0x10,       // Inactivity Test
    Xudt = 0x11,     // Extended Unitdata
    Xudts = 0x12,    // Extended Unitdata Service
    Ludt = 0x13,     // Long Unitdata
    Ludts = 0x14,    // Long Unitdata Service
}

/// SCCP Endpoint
pub struct SccpEndpoint {
    /// M3UA endpoint
    m3ua: Arc<M3uaEndpoint>,
    /// Local subsystem number
    local_ssn: u8,
    /// Local point code
    local_pc: u32,
    /// Global Title Translator
    gtt: GlobalTitleTranslator,
    /// Connection references
    next_local_ref: AtomicU32,
    /// Active connections
    connections: Arc<RwLock<HashMap<u32, SccpConnection>>>,
}

/// SCCP Connection (for connection-oriented services)
#[derive(Debug)]
pub struct SccpConnection {
    pub local_ref: u32,
    pub remote_ref: Option<u32>,
    pub remote_pc: u32,
    pub state: ConnectionState,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Idle,
    ConnectionPending,
    Active,
    DisconnectPending,
}

impl SccpEndpoint {
    /// Create new SCCP endpoint
    pub fn new(m3ua: Arc<M3uaEndpoint>, local_ssn: u8) -> Self {
        let local_pc = m3ua.point_code();
        
        Self {
            m3ua,
            local_ssn,
            local_pc,
            gtt: GlobalTitleTranslator::new(),
            next_local_ref: AtomicU32::new(1),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Send connectionless message (UDT)
    #[instrument(skip(self, data), fields(called = ?called_party.subsystem_number, len = data.len()))]
    pub async fn send_udt(
        &self,
        called_party: &SccpAddress,
        calling_party: &SccpAddress,
        data: &[u8],
    ) -> Result<(), SccpError> {
        let msg = SccpMessage::Udt {
            protocol_class: ProtocolClass::CLASS_0,
            called_party: called_party.clone(),
            calling_party: calling_party.clone(),
            data: Bytes::copy_from_slice(data),
        };

        let encoded = msg.encode();
        
        // Get destination point code
        let dpc = self.gtt.translate(called_party)?;
        
        self.m3ua.send_data(dpc, self.local_pc, 3, &encoded).await?;
        
        debug!("Sent UDT to DPC={}", dpc);
        Ok(())
    }

    /// Send extended unitdata (XUDT)
    #[instrument(skip(self, data), fields(called = ?called_party.subsystem_number))]
    pub async fn send_xudt(
        &self,
        called_party: &SccpAddress,
        calling_party: &SccpAddress,
        data: &[u8],
        hop_counter: u8,
    ) -> Result<(), SccpError> {
        let msg = SccpMessage::Xudt {
            protocol_class: ProtocolClass::CLASS_0,
            hop_counter,
            called_party: called_party.clone(),
            calling_party: calling_party.clone(),
            data: Bytes::copy_from_slice(data),
            segmentation: None,
            importance: None,
        };

        let encoded = msg.encode();
        let dpc = self.gtt.translate(called_party)?;
        
        self.m3ua.send_data(dpc, self.local_pc, 3, &encoded).await?;
        
        Ok(())
    }

    /// Receive SCCP message
    #[instrument(skip(self))]
    pub async fn recv(&self) -> Result<SccpMessage, SccpError> {
        let protocol_data = self.m3ua.recv_data().await?;
        
        if protocol_data.si != 3 {
            return Err(SccpError::InvalidMessage(
                format!("Expected SI=3, got SI={}", protocol_data.si)
            ));
        }

        SccpMessage::decode(&protocol_data.data)
    }

    /// Create local calling address
    pub fn local_address(&self) -> SccpAddress {
        SccpAddress::from_ssn_pc(self.local_ssn, self.local_pc)
    }

    /// Get next local reference
    fn next_local_reference(&self) -> u32 {
        self.next_local_ref.fetch_add(1, Ordering::Relaxed)
    }
}
