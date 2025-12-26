//! ASN.1 BER Encoding/Decoding utilities

use bytes::{BytesMut, BufMut};

/// Encode a tagged value (TLV)
pub fn encode_tagged(buf: &mut BytesMut, tag: u8, value: &[u8]) {
    buf.put_u8(tag);
    encode_length(buf, value.len());
    buf.put_slice(value);
}

/// Encode length in BER format
pub fn encode_length(buf: &mut BytesMut, len: usize) {
    if len < 128 {
        buf.put_u8(len as u8);
    } else if len < 256 {
        buf.put_u8(0x81);
        buf.put_u8(len as u8);
    } else if len < 65536 {
        buf.put_u8(0x82);
        buf.put_u16(len as u16);
    } else {
        buf.put_u8(0x84);
        buf.put_u32(len as u32);
    }
}

/// Encode an integer
pub fn encode_integer(buf: &mut BytesMut, tag: u8, value: i32) {
    let bytes = integer_to_bytes(value);
    encode_tagged(buf, tag, &bytes);
}

/// Convert integer to minimal byte representation
fn integer_to_bytes(value: i32) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }
    
    let bytes = value.to_be_bytes();
    let mut start = 0;
    
    // Skip leading zeros (or 0xFF for negative)
    if value > 0 {
        while start < 3 && bytes[start] == 0 {
            start += 1;
        }
        // Ensure positive numbers don't look negative
        if bytes[start] & 0x80 != 0 {
            let mut result = vec![0];
            result.extend_from_slice(&bytes[start..]);
            return result;
        }
    } else {
        while start < 3 && bytes[start] == 0xFF && (bytes[start + 1] & 0x80) != 0 {
            start += 1;
        }
    }
    
    bytes[start..].to_vec()
}

/// Decode a TLV structure
pub fn decode_tagged(data: &[u8]) -> Option<(u8, Vec<u8>)> {
    if data.is_empty() {
        return None;
    }
    
    let tag = data[0];
    let (length, header_len) = decode_length(&data[1..])?;
    
    if data.len() < 1 + header_len + length {
        return None;
    }
    
    let value = data[1 + header_len..1 + header_len + length].to_vec();
    Some((tag, value))
}

/// Decode BER length
pub fn decode_length(data: &[u8]) -> Option<(usize, usize)> {
    if data.is_empty() {
        return None;
    }
    
    let first = data[0];
    
    if first < 128 {
        Some((first as usize, 1))
    } else {
        let num_bytes = (first & 0x7F) as usize;
        if data.len() < 1 + num_bytes {
            return None;
        }
        
        let mut length = 0usize;
        for i in 0..num_bytes {
            length = (length << 8) | (data[1 + i] as usize);
        }
        
        Some((length, 1 + num_bytes))
    }
}

/// Get total TLV length
pub fn tlv_length(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }
    
    if let Some((length, header_len)) = decode_length(&data[1..]) {
        1 + header_len + length
    } else {
        0
    }
}

/// Encode OID
pub fn encode_oid(oid: &[u32]) -> Vec<u8> {
    let mut result = vec![0x06]; // OID tag
    
    if oid.len() < 2 {
        result.push(0);
        return result;
    }
    
    let mut content = Vec::new();
    
    // First two components
    content.push((oid[0] * 40 + oid[1]) as u8);
    
    // Remaining components
    for &component in &oid[2..] {
        encode_oid_component(&mut content, component);
    }
    
    result.push(content.len() as u8);
    result.extend(content);
    result
}

fn encode_oid_component(buf: &mut Vec<u8>, value: u32) {
    if value < 128 {
        buf.push(value as u8);
    } else {
        let mut bytes = Vec::new();
        let mut v = value;
        
        bytes.push((v & 0x7F) as u8);
        v >>= 7;
        
        while v > 0 {
            bytes.push(((v & 0x7F) | 0x80) as u8);
            v >>= 7;
        }
        
        bytes.reverse();
        buf.extend(bytes);
    }
}

/// Decode OID
pub fn decode_oid(data: &[u8]) -> Option<Vec<u32>> {
    if data.is_empty() {
        return None;
    }
    
    let mut oid = Vec::new();
    
    // First byte encodes first two components
    oid.push((data[0] / 40) as u32);
    oid.push((data[0] % 40) as u32);
    
    let mut i = 1;
    while i < data.len() {
        let (component, len) = decode_oid_component(&data[i..])?;
        oid.push(component);
        i += len;
    }
    
    Some(oid)
}

fn decode_oid_component(data: &[u8]) -> Option<(u32, usize)> {
    let mut value = 0u32;
    let mut len = 0;
    
    for &byte in data {
        len += 1;
        value = (value << 7) | ((byte & 0x7F) as u32);
        
        if byte & 0x80 == 0 {
            return Some((value, len));
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_encoding() {
        let mut buf = BytesMut::new();
        encode_length(&mut buf, 10);
        assert_eq!(&buf[..], &[10]);
        
        buf.clear();
        encode_length(&mut buf, 200);
        assert_eq!(&buf[..], &[0x81, 200]);
        
        buf.clear();
        encode_length(&mut buf, 1000);
        assert_eq!(&buf[..], &[0x82, 0x03, 0xE8]);
    }

    #[test]
    fn test_integer_encoding() {
        let mut buf = BytesMut::new();
        encode_integer(&mut buf, 0x02, 5);
        assert_eq!(&buf[..], &[0x02, 0x01, 0x05]);
    }
}
