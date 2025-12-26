//! MAP SMS Operations Encoding

use crate::types::{SmRpDa, SmRpOa, RoutingInfo};
use crate::errors::MapError;
use bytes::{BytesMut, BufMut};

/// MAP SMS Operation Enum
#[derive(Debug, Clone)]
pub enum MapSmsOperation {
    SendRoutingInfoForSm {
        msisdn: String,
        sm_rp_pri: bool,
        service_centre_address: String,
    },
    MoForwardShortMessage {
        sm_rp_da: SmRpDa,
        sm_rp_oa: SmRpOa,
        sm_rp_ui: Vec<u8>,
    },
    MtForwardShortMessage {
        sm_rp_da: SmRpDa,
        sm_rp_oa: SmRpOa,
        sm_rp_ui: Vec<u8>,
        more_messages_to_send: bool,
    },
}

/// Encode SRI-SM Request
pub fn encode_sri_sm_request(
    msisdn: &str,
    service_centre: &str,
    sm_rp_pri: bool,
) -> Vec<u8> {
    let mut buf = BytesMut::with_capacity(64);
    
    // SEQUENCE tag
    buf.put_u8(0x30);
    let content_start = buf.len();
    buf.put_u8(0x00); // Placeholder for length
    
    // MSISDN [0] ISDN-AddressString
    let msisdn_bcd = encode_tbcd(msisdn);
    buf.put_u8(0x80); // Context tag 0
    buf.put_u8(msisdn_bcd.len() as u8);
    buf.put_slice(&msisdn_bcd);
    
    // SM-RP-PRI [1] BOOLEAN
    buf.put_u8(0x81); // Context tag 1
    buf.put_u8(0x01);
    buf.put_u8(if sm_rp_pri { 0xFF } else { 0x00 });
    
    // ServiceCentreAddress [2] AddressString
    let sc_bcd = encode_tbcd(service_centre);
    buf.put_u8(0x82); // Context tag 2
    buf.put_u8(sc_bcd.len() as u8);
    buf.put_slice(&sc_bcd);
    
    // Update length
    let content_len = buf.len() - content_start - 1;
    buf[content_start] = content_len as u8;
    
    buf.to_vec()
}

/// Decode SRI-SM Response
pub fn decode_sri_sm_response(data: &[u8]) -> Result<RoutingInfo, MapError> {
    // Simplified parsing - in production, use proper ASN.1 decoder
    // Response contains IMSI and network node (MSC) number
    
    if data.len() < 10 {
        return Err(MapError::SystemFailure);
    }
    
    // Extract IMSI (first OCTET STRING after SEQUENCE)
    let mut offset = 2; // Skip SEQUENCE tag and length
    
    let imsi_len = data.get(offset + 1).copied().unwrap_or(0) as usize;
    let imsi = if offset + 2 + imsi_len <= data.len() {
        decode_tbcd(&data[offset + 2..offset + 2 + imsi_len])
    } else {
        "".to_string()
    };
    
    offset += 2 + imsi_len;
    
    // Extract MSC number (next address field)
    let msc_len = data.get(offset + 1).copied().unwrap_or(0) as usize;
    let msc_number = if offset + 2 + msc_len <= data.len() {
        decode_tbcd(&data[offset + 2..offset + 2 + msc_len])
    } else {
        "".to_string()
    };
    
    Ok(RoutingInfo {
        imsi,
        msc_number,
        lmsi: None,
    })
}

/// Encode MO-Forward-SM
pub fn encode_mo_forward_sm(
    sm_rp_da: SmRpDa,
    sm_rp_oa: SmRpOa,
    sm_rp_ui: &[u8],
) -> Vec<u8> {
    let mut buf = BytesMut::with_capacity(128);
    
    buf.put_u8(0x30); // SEQUENCE
    let len_pos = buf.len();
    buf.put_u8(0x00); // Length placeholder
    
    // SM-RP-DA [0]
    encode_sm_rp_da(&mut buf, &sm_rp_da, 0xA0);
    
    // SM-RP-OA [1]
    encode_sm_rp_oa(&mut buf, &sm_rp_oa, 0xA1);
    
    // SM-RP-UI [2] SignalInfo
    buf.put_u8(0x82);
    buf.put_u8(sm_rp_ui.len() as u8);
    buf.put_slice(sm_rp_ui);
    
    let len = buf.len() - len_pos - 1;
    buf[len_pos] = len as u8;
    
    buf.to_vec()
}

/// Encode MT-Forward-SM
pub fn encode_mt_forward_sm(
    sm_rp_da: SmRpDa,
    sm_rp_oa: SmRpOa,
    sm_rp_ui: &[u8],
    more_messages: bool,
) -> Vec<u8> {
    let mut buf = BytesMut::with_capacity(128);
    
    buf.put_u8(0x30); // SEQUENCE
    let len_pos = buf.len();
    buf.put_u8(0x00); // Length placeholder
    
    // SM-RP-DA [0]
    encode_sm_rp_da(&mut buf, &sm_rp_da, 0xA0);
    
    // SM-RP-OA [1]
    encode_sm_rp_oa(&mut buf, &sm_rp_oa, 0xA1);
    
    // SM-RP-UI [2] SignalInfo
    buf.put_u8(0x82);
    buf.put_u8(sm_rp_ui.len() as u8);
    buf.put_slice(sm_rp_ui);
    
    // More-Messages-To-Send [3] BOOLEAN OPTIONAL
    if more_messages {
        buf.put_u8(0x83);
        buf.put_u8(0x01);
        buf.put_u8(0xFF);
    }
    
    let len = buf.len() - len_pos - 1;
    buf[len_pos] = len as u8;
    
    buf.to_vec()
}

fn encode_sm_rp_da(buf: &mut BytesMut, da: &SmRpDa, tag: u8) {
    buf.put_u8(tag);
    let len_pos = buf.len();
    buf.put_u8(0x00);
    
    match da {
        SmRpDa::Imsi(imsi) => {
            let bcd = encode_tbcd(imsi);
            buf.put_u8(0x80);
            buf.put_u8(bcd.len() as u8);
            buf.put_slice(&bcd);
        }
        SmRpDa::Lmsi(lmsi) => {
            buf.put_u8(0x81);
            buf.put_u8(lmsi.len() as u8);
            buf.put_slice(lmsi);
        }
        SmRpDa::ServiceCentreAddress(addr) => {
            let bcd = encode_tbcd(addr);
            buf.put_u8(0x82);
            buf.put_u8(bcd.len() as u8);
            buf.put_slice(&bcd);
        }
        SmRpDa::NoSmRpDa => {
            buf.put_u8(0x85);
            buf.put_u8(0x00);
        }
    }
    
    let len = buf.len() - len_pos - 1;
    buf[len_pos] = len as u8;
}

fn encode_sm_rp_oa(buf: &mut BytesMut, oa: &SmRpOa, tag: u8) {
    buf.put_u8(tag);
    let len_pos = buf.len();
    buf.put_u8(0x00);
    
    match oa {
        SmRpOa::Msisdn(msisdn) => {
            let bcd = encode_tbcd(msisdn);
            buf.put_u8(0x82);
            buf.put_u8(bcd.len() as u8);
            buf.put_slice(&bcd);
        }
        SmRpOa::ServiceCentreAddress(addr) => {
            let bcd = encode_tbcd(addr);
            buf.put_u8(0x84);
            buf.put_u8(bcd.len() as u8);
            buf.put_slice(&bcd);
        }
        SmRpOa::NoSmRpOa => {
            buf.put_u8(0x85);
            buf.put_u8(0x00);
        }
    }
    
    let len = buf.len() - len_pos - 1;
    buf[len_pos] = len as u8;
}

/// Encode to TBCD (Telephony BCD)
fn encode_tbcd(number: &str) -> Vec<u8> {
    let digits: Vec<u8> = number
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '*' || *c == '#')
        .map(|c| match c {
            '*' => 0x0A,
            '#' => 0x0B,
            d => d.to_digit(10).unwrap() as u8,
        })
        .collect();
    
    // TON/NPI byte (International, E.164)
    let mut result = vec![0x91];
    
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

/// Decode from TBCD
fn decode_tbcd(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }
    
    let mut result = String::new();
    
    // Skip TON/NPI byte if present
    let start = if data[0] & 0x80 != 0 { 1 } else { 0 };
    
    for &byte in &data[start..] {
        let low = byte & 0x0F;
        let high = (byte >> 4) & 0x0F;
        
        if low < 10 {
            result.push(char::from_digit(low as u32, 10).unwrap());
        } else if low == 0x0A {
            result.push('*');
        } else if low == 0x0B {
            result.push('#');
        }
        
        if high < 10 {
            result.push(char::from_digit(high as u32, 10).unwrap());
        } else if high == 0x0A {
            result.push('*');
        } else if high == 0x0B {
            result.push('#');
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tbcd_encoding() {
        let encoded = encode_tbcd("1234567890");
        assert_eq!(encoded[0], 0x91); // TON/NPI
        assert_eq!(encoded[1], 0x21); // 1, 2
        assert_eq!(encoded[2], 0x43); // 3, 4
    }

    #[test]
    fn test_tbcd_decoding() {
        let data = vec![0x91, 0x21, 0x43, 0x65];
        let decoded = decode_tbcd(&data);
        assert_eq!(decoded, "123456");  // 3 bytes = 6 digits (0x21=12, 0x43=34, 0x65=56)
    }
}
