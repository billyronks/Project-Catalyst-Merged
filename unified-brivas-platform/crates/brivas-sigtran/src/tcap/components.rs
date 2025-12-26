//! TCAP Components

use bytes::{BytesMut, BufMut};
use super::asn1;

/// TCAP Component Tags
#[repr(u8)]
pub enum ComponentTag {
    Invoke = 0xA1,
    ReturnResultLast = 0xA2,
    ReturnError = 0xA3,
    Reject = 0xA4,
    ReturnResultNotLast = 0xA7,
}

/// TCAP Component
#[derive(Debug, Clone)]
pub enum Component {
    Invoke {
        invoke_id: i32,
        linked_id: Option<i32>,
        operation_code: i32,
        parameter: Option<Vec<u8>>,
    },
    ReturnResultLast {
        invoke_id: i32,
        operation_code: Option<i32>,
        parameter: Option<Vec<u8>>,
    },
    ReturnResultNotLast {
        invoke_id: i32,
        operation_code: Option<i32>,
        parameter: Option<Vec<u8>>,
    },
    ReturnError {
        invoke_id: i32,
        error_code: i32,
        parameter: Option<Vec<u8>>,
    },
    Reject {
        invoke_id: Option<i32>,
        problem_code: u8,
    },
}

impl Component {
    /// Encode component to ASN.1
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(256);

        match self {
            Self::Invoke { invoke_id, linked_id, operation_code, parameter } => {
                let mut content = BytesMut::new();
                
                // Invoke ID
                asn1::encode_integer(&mut content, 0x02, *invoke_id);
                
                // Linked ID (optional)
                if let Some(lid) = linked_id {
                    asn1::encode_integer(&mut content, 0x80, *lid);
                }
                
                // Operation code
                asn1::encode_integer(&mut content, 0x02, *operation_code);
                
                // Parameter (optional)
                if let Some(param) = parameter {
                    content.put_slice(param);
                }
                
                asn1::encode_tagged(&mut buf, ComponentTag::Invoke as u8, &content);
            }
            Self::ReturnResultLast { invoke_id, operation_code, parameter } => {
                let mut content = BytesMut::new();
                
                asn1::encode_integer(&mut content, 0x02, *invoke_id);
                
                // Result (sequence of opcode + parameter)
                if operation_code.is_some() || parameter.is_some() {
                    let mut result_content = BytesMut::new();
                    
                    if let Some(op) = operation_code {
                        asn1::encode_integer(&mut result_content, 0x02, *op);
                    }
                    
                    if let Some(param) = parameter {
                        result_content.put_slice(param);
                    }
                    
                    asn1::encode_tagged(&mut content, 0x30, &result_content);
                }
                
                asn1::encode_tagged(&mut buf, ComponentTag::ReturnResultLast as u8, &content);
            }
            Self::ReturnResultNotLast { invoke_id, operation_code, parameter } => {
                let mut content = BytesMut::new();
                
                asn1::encode_integer(&mut content, 0x02, *invoke_id);
                
                if operation_code.is_some() || parameter.is_some() {
                    let mut result_content = BytesMut::new();
                    
                    if let Some(op) = operation_code {
                        asn1::encode_integer(&mut result_content, 0x02, *op);
                    }
                    
                    if let Some(param) = parameter {
                        result_content.put_slice(param);
                    }
                    
                    asn1::encode_tagged(&mut content, 0x30, &result_content);
                }
                
                asn1::encode_tagged(&mut buf, ComponentTag::ReturnResultNotLast as u8, &content);
            }
            Self::ReturnError { invoke_id, error_code, parameter } => {
                let mut content = BytesMut::new();
                
                asn1::encode_integer(&mut content, 0x02, *invoke_id);
                asn1::encode_integer(&mut content, 0x02, *error_code);
                
                if let Some(param) = parameter {
                    content.put_slice(param);
                }
                
                asn1::encode_tagged(&mut buf, ComponentTag::ReturnError as u8, &content);
            }
            Self::Reject { invoke_id, problem_code } => {
                let mut content = BytesMut::new();
                
                if let Some(iid) = invoke_id {
                    asn1::encode_integer(&mut content, 0x02, *iid);
                } else {
                    content.put_u8(0x05); // NULL
                    content.put_u8(0x00);
                }
                
                asn1::encode_integer(&mut content, 0x02, *problem_code as i32);
                
                asn1::encode_tagged(&mut buf, ComponentTag::Reject as u8, &content);
            }
        }

        buf
    }

    /// Decode component from ASN.1
    pub fn decode(tag: u8, data: &[u8]) -> Option<Self> {
        match tag {
            0xA1 => { // Invoke
                let mut offset = 0;
                
                // Parse invoke ID
                let (_, invoke_id_bytes) = asn1::decode_tagged(&data[offset..])?;
                let invoke_id = parse_integer(&invoke_id_bytes)?;
                offset += asn1::tlv_length(&data[offset..]);
                
                // Check for linked ID (context tag 0x80)
                let linked_id = if offset < data.len() && data[offset] == 0x80 {
                    let (_, lid_bytes) = asn1::decode_tagged(&data[offset..])?;
                    offset += asn1::tlv_length(&data[offset..]);
                    Some(parse_integer(&lid_bytes)?)
                } else {
                    None
                };
                
                // Parse operation code
                if offset >= data.len() {
                    return None;
                }
                let (_, op_bytes) = asn1::decode_tagged(&data[offset..])?;
                let operation_code = parse_integer(&op_bytes)?;
                offset += asn1::tlv_length(&data[offset..]);
                
                // Remaining is parameter
                let parameter = if offset < data.len() {
                    Some(data[offset..].to_vec())
                } else {
                    None
                };
                
                Some(Self::Invoke {
                    invoke_id,
                    linked_id,
                    operation_code,
                    parameter,
                })
            }
            0xA2 => { // ReturnResultLast
                let mut offset = 0;
                
                let (_, invoke_id_bytes) = asn1::decode_tagged(&data[offset..])?;
                let invoke_id = parse_integer(&invoke_id_bytes)?;
                offset += asn1::tlv_length(&data[offset..]);
                
                let (operation_code, parameter) = if offset < data.len() {
                    // Parse result sequence
                    let (_, result_bytes) = asn1::decode_tagged(&data[offset..])?;
                    
                    if !result_bytes.is_empty() {
                        let (_, op_bytes) = asn1::decode_tagged(&result_bytes)?;
                        let op = parse_integer(&op_bytes)?;
                        let param_offset = asn1::tlv_length(&result_bytes);
                        let param = if param_offset < result_bytes.len() {
                            Some(result_bytes[param_offset..].to_vec())
                        } else {
                            None
                        };
                        (Some(op), param)
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };
                
                Some(Self::ReturnResultLast {
                    invoke_id,
                    operation_code,
                    parameter,
                })
            }
            0xA3 => { // ReturnError
                let mut offset = 0;
                
                let (_, invoke_id_bytes) = asn1::decode_tagged(&data[offset..])?;
                let invoke_id = parse_integer(&invoke_id_bytes)?;
                offset += asn1::tlv_length(&data[offset..]);
                
                let (_, error_bytes) = asn1::decode_tagged(&data[offset..])?;
                let error_code = parse_integer(&error_bytes)?;
                offset += asn1::tlv_length(&data[offset..]);
                
                let parameter = if offset < data.len() {
                    Some(data[offset..].to_vec())
                } else {
                    None
                };
                
                Some(Self::ReturnError {
                    invoke_id,
                    error_code,
                    parameter,
                })
            }
            0xA4 => { // Reject
                Some(Self::Reject {
                    invoke_id: None,
                    problem_code: 0,
                })
            }
            _ => None,
        }
    }

    /// Get invoke ID
    pub fn invoke_id(&self) -> Option<i32> {
        match self {
            Self::Invoke { invoke_id, .. } => Some(*invoke_id),
            Self::ReturnResultLast { invoke_id, .. } => Some(*invoke_id),
            Self::ReturnResultNotLast { invoke_id, .. } => Some(*invoke_id),
            Self::ReturnError { invoke_id, .. } => Some(*invoke_id),
            Self::Reject { invoke_id, .. } => *invoke_id,
        }
    }
}

fn parse_integer(data: &[u8]) -> Option<i32> {
    if data.is_empty() {
        return Some(0);
    }
    
    let mut result: i32 = 0;
    for &byte in data {
        result = (result << 8) | (byte as i32);
    }
    Some(result)
}
