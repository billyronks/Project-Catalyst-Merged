//! SCCP Message Types

use super::address::SccpAddress;
use super::MessageType;
use crate::errors::SccpError;
use crate::types::{ProtocolClass, Segmentation};
use bytes::{Bytes, BytesMut, Buf, BufMut};

/// SCCP Message
#[derive(Debug, Clone)]
pub enum SccpMessage {
    /// Unitdata (connectionless)
    Udt {
        protocol_class: ProtocolClass,
        called_party: SccpAddress,
        calling_party: SccpAddress,
        data: Bytes,
    },
    /// Extended Unitdata
    Xudt {
        protocol_class: ProtocolClass,
        hop_counter: u8,
        called_party: SccpAddress,
        calling_party: SccpAddress,
        data: Bytes,
        segmentation: Option<Segmentation>,
        importance: Option<u8>,
    },
    /// Connection Request
    Cr {
        source_local_reference: u32,
        protocol_class: ProtocolClass,
        called_party: SccpAddress,
        credit: Option<u8>,
        calling_party: Option<SccpAddress>,
        data: Option<Bytes>,
    },
    /// Connection Confirm
    Cc {
        destination_local_reference: u32,
        source_local_reference: u32,
        protocol_class: ProtocolClass,
        credit: Option<u8>,
        called_party: Option<SccpAddress>,
        data: Option<Bytes>,
    },
    /// Released
    Rlsd {
        destination_local_reference: u32,
        source_local_reference: u32,
        release_cause: u8,
        data: Option<Bytes>,
    },
    /// Release Complete
    Rlc {
        destination_local_reference: u32,
        source_local_reference: u32,
    },
    /// Data Form 1
    Dt1 {
        destination_local_reference: u32,
        segmenting: bool,
        data: Bytes,
    },
}

impl SccpMessage {
    /// Get message type
    pub fn message_type(&self) -> MessageType {
        match self {
            Self::Udt { .. } => MessageType::Udt,
            Self::Xudt { .. } => MessageType::Xudt,
            Self::Cr { .. } => MessageType::Cr,
            Self::Cc { .. } => MessageType::Cc,
            Self::Rlsd { .. } => MessageType::Rlsd,
            Self::Rlc { .. } => MessageType::Rlc,
            Self::Dt1 { .. } => MessageType::Dt1,
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(256);

        match self {
            Self::Udt { protocol_class, called_party, calling_party, data } => {
                buf.put_u8(MessageType::Udt as u8);
                buf.put_u8(protocol_class.encode());
                
                // Variable part pointers
                let called_addr = called_party.encode();
                let calling_addr = calling_party.encode();
                
                let ptr_called = 3u8;
                let ptr_calling = ptr_called + 1 + called_addr.len() as u8;
                let ptr_data = ptr_calling + 1 + calling_addr.len() as u8;
                
                buf.put_u8(ptr_called);
                buf.put_u8(ptr_calling);
                buf.put_u8(ptr_data);
                
                // Variable parts
                buf.put_u8(called_addr.len() as u8);
                buf.put_slice(&called_addr);
                
                buf.put_u8(calling_addr.len() as u8);
                buf.put_slice(&calling_addr);
                
                buf.put_u8(data.len() as u8);
                buf.put_slice(data);
            }
            Self::Xudt { protocol_class, hop_counter, called_party, calling_party, data, segmentation, importance } => {
                buf.put_u8(MessageType::Xudt as u8);
                buf.put_u8(protocol_class.encode());
                buf.put_u8(*hop_counter);
                
                // Similar to UDT but with optional parameters
                let called_addr = called_party.encode();
                let calling_addr = calling_party.encode();
                
                let ptr_called = 4u8;
                let ptr_calling = ptr_called + 1 + called_addr.len() as u8;
                let ptr_data = ptr_calling + 1 + calling_addr.len() as u8;
                let ptr_optional = 0u8; // No optional for now
                
                buf.put_u8(ptr_called);
                buf.put_u8(ptr_calling);
                buf.put_u8(ptr_data);
                buf.put_u8(ptr_optional);
                
                buf.put_u8(called_addr.len() as u8);
                buf.put_slice(&called_addr);
                
                buf.put_u8(calling_addr.len() as u8);
                buf.put_slice(&calling_addr);
                
                buf.put_u8(data.len() as u8);
                buf.put_slice(data);
            }
            Self::Cr { source_local_reference, protocol_class, called_party, credit, calling_party, data } => {
                buf.put_u8(MessageType::Cr as u8);
                
                // Source local reference (3 bytes little endian)
                buf.put_u8(*source_local_reference as u8);
                buf.put_u8((*source_local_reference >> 8) as u8);
                buf.put_u8((*source_local_reference >> 16) as u8);
                
                buf.put_u8(protocol_class.encode());
                
                let called_addr = called_party.encode();
                buf.put_u8(2); // Pointer to called party
                buf.put_u8(called_addr.len() as u8);
                buf.put_slice(&called_addr);
            }
            Self::Cc { destination_local_reference, source_local_reference, protocol_class, .. } => {
                buf.put_u8(MessageType::Cc as u8);
                
                // Destination local reference
                buf.put_u8(*destination_local_reference as u8);
                buf.put_u8((*destination_local_reference >> 8) as u8);
                buf.put_u8((*destination_local_reference >> 16) as u8);
                
                // Source local reference
                buf.put_u8(*source_local_reference as u8);
                buf.put_u8((*source_local_reference >> 8) as u8);
                buf.put_u8((*source_local_reference >> 16) as u8);
                
                buf.put_u8(protocol_class.encode());
            }
            Self::Rlsd { destination_local_reference, source_local_reference, release_cause, .. } => {
                buf.put_u8(MessageType::Rlsd as u8);
                
                buf.put_u8(*destination_local_reference as u8);
                buf.put_u8((*destination_local_reference >> 8) as u8);
                buf.put_u8((*destination_local_reference >> 16) as u8);
                
                buf.put_u8(*source_local_reference as u8);
                buf.put_u8((*source_local_reference >> 8) as u8);
                buf.put_u8((*source_local_reference >> 16) as u8);
                
                buf.put_u8(*release_cause);
            }
            Self::Rlc { destination_local_reference, source_local_reference } => {
                buf.put_u8(MessageType::Rlc as u8);
                
                buf.put_u8(*destination_local_reference as u8);
                buf.put_u8((*destination_local_reference >> 8) as u8);
                buf.put_u8((*destination_local_reference >> 16) as u8);
                
                buf.put_u8(*source_local_reference as u8);
                buf.put_u8((*source_local_reference >> 8) as u8);
                buf.put_u8((*source_local_reference >> 16) as u8);
            }
            Self::Dt1 { destination_local_reference, segmenting, data } => {
                buf.put_u8(MessageType::Dt1 as u8);
                
                buf.put_u8(*destination_local_reference as u8);
                buf.put_u8((*destination_local_reference >> 8) as u8);
                buf.put_u8((*destination_local_reference >> 16) as u8);
                
                buf.put_u8(if *segmenting { 0x01 } else { 0x00 });
                buf.put_u8(1); // Pointer
                buf.put_u8(data.len() as u8);
                buf.put_slice(data);
            }
        }

        buf
    }

    /// Decode from bytes
    pub fn decode(data: &Bytes) -> Result<Self, SccpError> {
        if data.is_empty() {
            return Err(SccpError::InvalidMessage("Empty message".to_string()));
        }

        let mut buf = data.clone();
        let msg_type = buf.get_u8();

        match msg_type {
            0x09 => { // UDT
                let pc = ProtocolClass::decode(buf.get_u8());
                
                let ptr_called = buf.get_u8() as usize;
                let ptr_calling = buf.get_u8() as usize;
                let ptr_data = buf.get_u8() as usize;
                
                // Parse variable parts from the original data
                let base = 5; // After fixed part
                
                let called_len = data[base + ptr_called - 3] as usize;
                let called_data = Bytes::copy_from_slice(&data[base + ptr_called - 3 + 1..base + ptr_called - 3 + 1 + called_len]);
                let called_party = SccpAddress::decode(called_data)
                    .ok_or_else(|| SccpError::AddressError("Invalid called party".to_string()))?;
                
                let calling_offset = base + ptr_calling - 2;
                let calling_len = data[calling_offset] as usize;
                let calling_data = Bytes::copy_from_slice(&data[calling_offset + 1..calling_offset + 1 + calling_len]);
                let calling_party = SccpAddress::decode(calling_data)
                    .ok_or_else(|| SccpError::AddressError("Invalid calling party".to_string()))?;
                
                let data_offset = base + ptr_data - 1;
                let data_len = data[data_offset] as usize;
                let user_data = Bytes::copy_from_slice(&data[data_offset + 1..data_offset + 1 + data_len]);
                
                Ok(Self::Udt {
                    protocol_class: pc,
                    called_party,
                    calling_party,
                    data: user_data,
                })
            }
            0x11 => { // XUDT
                let pc = ProtocolClass::decode(buf.get_u8());
                let hop_counter = buf.get_u8();
                
                // Similar parsing as UDT
                Ok(Self::Xudt {
                    protocol_class: pc,
                    hop_counter,
                    called_party: SccpAddress::from_ssn_pc(0, 0), // Simplified
                    calling_party: SccpAddress::from_ssn_pc(0, 0),
                    data: buf.copy_to_bytes(buf.remaining()),
                    segmentation: None,
                    importance: None,
                })
            }
            _ => Err(SccpError::InvalidMessage(format!("Unknown message type: 0x{:02X}", msg_type))),
        }
    }
}
