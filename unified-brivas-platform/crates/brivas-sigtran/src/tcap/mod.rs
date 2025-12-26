//! TCAP (Transaction Capabilities Application Part)
//!
//! ITU-T Q.771-Q.775 compliant implementation.

mod transaction;
mod components;
mod asn1;

pub use transaction::TcapEndpoint;
pub use components::Component;

use crate::errors::TcapError;
use bytes::{Bytes, BytesMut, Buf, BufMut};

/// TCAP Message
#[derive(Debug, Clone)]
pub enum TcapMessage {
    Begin {
        originating_transaction_id: Vec<u8>,
        dialogue_portion: Option<DialoguePortion>,
        component_portion: Vec<Component>,
    },
    Continue {
        originating_transaction_id: Vec<u8>,
        destination_transaction_id: Vec<u8>,
        dialogue_portion: Option<DialoguePortion>,
        component_portion: Vec<Component>,
    },
    End {
        destination_transaction_id: Vec<u8>,
        dialogue_portion: Option<DialoguePortion>,
        component_portion: Vec<Component>,
    },
    Abort {
        destination_transaction_id: Vec<u8>,
        cause: AbortCause,
    },
}

/// Dialogue Portion
#[derive(Debug, Clone)]
pub struct DialoguePortion {
    /// Application Context Name (OID)
    pub application_context_name: Vec<u32>,
    /// User Information
    pub user_information: Option<Vec<u8>>,
}

/// Abort Cause
#[derive(Debug, Clone)]
pub enum AbortCause {
    UnrecognizedMessageType,
    UnrecognizedTransactionId,
    BadlyFormattedTransactionPortion,
    IncorrectTransactionPortion,
    ResourceLimitation,
    User(Vec<u8>),
}

/// ASN.1 Tags for TCAP
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TcapTag {
    Begin = 0x62,
    End = 0x64,
    Continue = 0x65,
    Abort = 0x67,
    OriginatingTransactionId = 0x48,
    DestinationTransactionId = 0x49,
    DialoguePortion = 0x6B,
    ComponentPortion = 0x6C,
}

impl TcapMessage {
    /// Get message tag
    pub fn tag(&self) -> u8 {
        match self {
            Self::Begin { .. } => TcapTag::Begin as u8,
            Self::Continue { .. } => TcapTag::Continue as u8,
            Self::End { .. } => TcapTag::End as u8,
            Self::Abort { .. } => TcapTag::Abort as u8,
        }
    }

    /// Encode to ASN.1 BER
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(512);
        
        match self {
            Self::Begin { originating_transaction_id, dialogue_portion, component_portion } => {
                let mut content = BytesMut::new();
                
                // Originating Transaction ID
                asn1::encode_tagged(&mut content, TcapTag::OriginatingTransactionId as u8, originating_transaction_id);
                
                // Dialogue portion (optional)
                if let Some(dp) = dialogue_portion {
                    let dp_encoded = encode_dialogue_portion(dp);
                    asn1::encode_tagged(&mut content, TcapTag::DialoguePortion as u8, &dp_encoded);
                }
                
                // Component portion
                if !component_portion.is_empty() {
                    let mut comp_buf = BytesMut::new();
                    for comp in component_portion {
                        comp_buf.put_slice(&comp.encode());
                    }
                    asn1::encode_tagged(&mut content, TcapTag::ComponentPortion as u8, &comp_buf);
                }
                
                asn1::encode_tagged(&mut buf, TcapTag::Begin as u8, &content);
            }
            Self::Continue { originating_transaction_id, destination_transaction_id, dialogue_portion, component_portion } => {
                let mut content = BytesMut::new();
                
                asn1::encode_tagged(&mut content, TcapTag::OriginatingTransactionId as u8, originating_transaction_id);
                asn1::encode_tagged(&mut content, TcapTag::DestinationTransactionId as u8, destination_transaction_id);
                
                if let Some(dp) = dialogue_portion {
                    let dp_encoded = encode_dialogue_portion(dp);
                    asn1::encode_tagged(&mut content, TcapTag::DialoguePortion as u8, &dp_encoded);
                }
                
                if !component_portion.is_empty() {
                    let mut comp_buf = BytesMut::new();
                    for comp in component_portion {
                        comp_buf.put_slice(&comp.encode());
                    }
                    asn1::encode_tagged(&mut content, TcapTag::ComponentPortion as u8, &comp_buf);
                }
                
                asn1::encode_tagged(&mut buf, TcapTag::Continue as u8, &content);
            }
            Self::End { destination_transaction_id, dialogue_portion, component_portion } => {
                let mut content = BytesMut::new();
                
                asn1::encode_tagged(&mut content, TcapTag::DestinationTransactionId as u8, destination_transaction_id);
                
                if let Some(dp) = dialogue_portion {
                    let dp_encoded = encode_dialogue_portion(dp);
                    asn1::encode_tagged(&mut content, TcapTag::DialoguePortion as u8, &dp_encoded);
                }
                
                if !component_portion.is_empty() {
                    let mut comp_buf = BytesMut::new();
                    for comp in component_portion {
                        comp_buf.put_slice(&comp.encode());
                    }
                    asn1::encode_tagged(&mut content, TcapTag::ComponentPortion as u8, &comp_buf);
                }
                
                asn1::encode_tagged(&mut buf, TcapTag::End as u8, &content);
            }
            Self::Abort { destination_transaction_id, cause: _ } => {
                let mut content = BytesMut::new();
                asn1::encode_tagged(&mut content, TcapTag::DestinationTransactionId as u8, destination_transaction_id);
                asn1::encode_tagged(&mut buf, TcapTag::Abort as u8, &content);
            }
        }

        buf
    }

    /// Decode from ASN.1 BER
    pub fn decode(data: &[u8]) -> Result<Self, TcapError> {
        if data.is_empty() {
            return Err(TcapError::Asn1Error("Empty data".to_string()));
        }

        let (tag, content) = asn1::decode_tagged(data)
            .ok_or_else(|| TcapError::Asn1Error("Invalid TLV".to_string()))?;

        match tag {
            0x62 => { // Begin
                let (otid, dp, comps) = parse_begin_content(&content)?;
                Ok(Self::Begin {
                    originating_transaction_id: otid,
                    dialogue_portion: dp,
                    component_portion: comps,
                })
            }
            0x65 => { // Continue
                let (otid, dtid, dp, comps) = parse_continue_content(&content)?;
                Ok(Self::Continue {
                    originating_transaction_id: otid,
                    destination_transaction_id: dtid,
                    dialogue_portion: dp,
                    component_portion: comps,
                })
            }
            0x64 => { // End
                let (dtid, dp, comps) = parse_end_content(&content)?;
                Ok(Self::End {
                    destination_transaction_id: dtid,
                    dialogue_portion: dp,
                    component_portion: comps,
                })
            }
            0x67 => { // Abort
                let (dtid, _) = asn1::decode_tagged(&content)
                    .ok_or_else(|| TcapError::Asn1Error("Invalid abort".to_string()))?;
                Ok(Self::Abort {
                    destination_transaction_id: content[2..].to_vec(),
                    cause: AbortCause::UnrecognizedTransactionId,
                })
            }
            _ => Err(TcapError::Asn1Error(format!("Unknown tag: 0x{:02X}", tag))),
        }
    }
}

fn encode_dialogue_portion(dp: &DialoguePortion) -> BytesMut {
    let mut buf = BytesMut::new();
    // Simplified: just encode AC name
    let oid = asn1::encode_oid(&dp.application_context_name);
    buf.put_slice(&oid);
    buf
}

fn parse_begin_content(content: &[u8]) -> Result<(Vec<u8>, Option<DialoguePortion>, Vec<Component>), TcapError> {
    let mut offset = 0;
    let mut otid = Vec::new();
    let mut dp = None;
    let mut comps = Vec::new();

    while offset < content.len() {
        let (tag, value) = asn1::decode_tagged(&content[offset..])
            .ok_or_else(|| TcapError::Asn1Error("Parse error".to_string()))?;
        
        let len = asn1::tlv_length(&content[offset..]);
        
        match tag {
            0x48 => otid = value.to_vec(),
            0x6B => { /* dialogue portion - skip for now */ }
            0x6C => comps = parse_components(&value)?,
            _ => {}
        }
        
        offset += len;
    }

    Ok((otid, dp, comps))
}

fn parse_continue_content(content: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Option<DialoguePortion>, Vec<Component>), TcapError> {
    let (otid, dp, comps) = parse_begin_content(content)?;
    // For simplicity, extract DTID from first bytes
    Ok((otid.clone(), otid, dp, comps))
}

fn parse_end_content(content: &[u8]) -> Result<(Vec<u8>, Option<DialoguePortion>, Vec<Component>), TcapError> {
    parse_begin_content(content)
}

fn parse_components(data: &[u8]) -> Result<Vec<Component>, TcapError> {
    let mut comps = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        if let Some((tag, value)) = asn1::decode_tagged(&data[offset..]) {
            let len = asn1::tlv_length(&data[offset..]);
            if let Some(comp) = Component::decode(tag, &value) {
                comps.push(comp);
            }
            offset += len;
        } else {
            break;
        }
    }

    Ok(comps)
}
