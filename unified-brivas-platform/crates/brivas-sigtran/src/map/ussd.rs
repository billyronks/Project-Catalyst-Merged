//! MAP USSD Operations

use crate::types::UssdResponse;
use crate::errors::MapError;
use super::encoding::{encode_ussd_string, decode_ussd_string};
use bytes::{BytesMut, BufMut};

/// MAP USSD Operation Enum
#[derive(Debug, Clone)]
pub enum MapUssdOperation {
    ProcessUnstructuredSsRequest {
        ussd_data_coding_scheme: u8,
        ussd_string: Vec<u8>,
        msisdn: Option<String>,
    },
    UnstructuredSsRequest {
        ussd_data_coding_scheme: u8,
        ussd_string: Vec<u8>,
        msisdn: String,
    },
    UnstructuredSsNotify {
        ussd_data_coding_scheme: u8,
        ussd_string: Vec<u8>,
    },
}

/// Encode ProcessUnstructuredSS-Request
pub fn encode_process_ussd_request(
    dcs: u8,
    ussd_string: &[u8],
    msisdn: Option<&str>,
) -> Vec<u8> {
    let mut buf = BytesMut::with_capacity(64);
    
    buf.put_u8(0x30); // SEQUENCE
    let len_pos = buf.len();
    buf.put_u8(0x00); // Length placeholder
    
    // ussd-DataCodingScheme [0] USSD-DataCodingScheme
    buf.put_u8(0x80); // Context tag 0
    buf.put_u8(0x01);
    buf.put_u8(dcs);
    
    // ussd-String [1] USSD-String
    buf.put_u8(0x81); // Context tag 1
    buf.put_u8(ussd_string.len() as u8);
    buf.put_slice(ussd_string);
    
    // msisdn [2] ISDN-AddressString OPTIONAL
    if let Some(ms) = msisdn {
        let bcd = encode_msisdn_bcd(ms);
        buf.put_u8(0x82);
        buf.put_u8(bcd.len() as u8);
        buf.put_slice(&bcd);
    }
    
    let len = buf.len() - len_pos - 1;
    buf[len_pos] = len as u8;
    
    buf.to_vec()
}

/// Encode UnstructuredSS-Request
pub fn encode_ussd_request(
    dcs: u8,
    ussd_string: &[u8],
    msisdn: &str,
) -> Vec<u8> {
    let mut buf = BytesMut::with_capacity(64);
    
    buf.put_u8(0x30); // SEQUENCE
    let len_pos = buf.len();
    buf.put_u8(0x00); // Length placeholder
    
    // ussd-DataCodingScheme USSD-DataCodingScheme
    buf.put_u8(0x04); // OCTET STRING
    buf.put_u8(0x01);
    buf.put_u8(dcs);
    
    // ussd-String USSD-String
    buf.put_u8(0x04); // OCTET STRING
    buf.put_u8(ussd_string.len() as u8);
    buf.put_slice(ussd_string);
    
    // msisdn ISDN-AddressString
    let bcd = encode_msisdn_bcd(msisdn);
    buf.put_u8(0x80);
    buf.put_u8(bcd.len() as u8);
    buf.put_slice(&bcd);
    
    let len = buf.len() - len_pos - 1;
    buf[len_pos] = len as u8;
    
    buf.to_vec()
}

/// Encode UnstructuredSS-Notify
pub fn encode_ussd_notify(dcs: u8, ussd_string: &[u8]) -> Vec<u8> {
    let mut buf = BytesMut::with_capacity(32);
    
    buf.put_u8(0x30); // SEQUENCE
    let len_pos = buf.len();
    buf.put_u8(0x00); // Length placeholder
    
    buf.put_u8(0x04); // OCTET STRING
    buf.put_u8(0x01);
    buf.put_u8(dcs);
    
    buf.put_u8(0x04); // OCTET STRING
    buf.put_u8(ussd_string.len() as u8);
    buf.put_slice(ussd_string);
    
    let len = buf.len() - len_pos - 1;
    buf[len_pos] = len as u8;
    
    buf.to_vec()
}

/// Decode USSD Response
pub fn decode_ussd_response(data: &[u8]) -> Result<UssdResponse, MapError> {
    if data.len() < 4 {
        return Err(MapError::SystemFailure);
    }
    
    let mut offset = 0;
    
    // Skip SEQUENCE header
    if data[offset] == 0x30 {
        offset += 2;
    }
    
    // Find DCS
    let dcs = if offset < data.len() && data[offset] == 0x04 {
        offset += 2;
        let d = data.get(offset).copied().unwrap_or(0x0F);
        offset += 1;
        d
    } else if offset < data.len() && data[offset] == 0x80 {
        offset += 2;
        let d = data.get(offset).copied().unwrap_or(0x0F);
        offset += 1;
        d
    } else {
        0x0F
    };
    
    // Find USSD string
    let ussd_string = if offset < data.len() {
        let tag = data[offset];
        if tag == 0x04 || tag == 0x81 {
            offset += 1;
            let len = data.get(offset).copied().unwrap_or(0) as usize;
            offset += 1;
            if offset + len <= data.len() {
                data[offset..offset + len].to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    Ok(UssdResponse {
        ussd_string,
        dcs,
        release: false,
    })
}

/// Encode MSISDN to BCD
fn encode_msisdn_bcd(msisdn: &str) -> Vec<u8> {
    let digits: Vec<u8> = msisdn
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect();
    
    // Length byte + TON/NPI
    let len_byte = 1 + (digits.len() + 1) / 2;
    let mut result = vec![len_byte as u8, 0x91]; // International, E.164
    
    for chunk in digits.chunks(2) {
        let byte = if chunk.len() == 2 {
            chunk[0] | (chunk[1] << 4)
        } else {
            chunk[0] | 0xF0
        };
        result.push(byte);
    }
    
    result
}
