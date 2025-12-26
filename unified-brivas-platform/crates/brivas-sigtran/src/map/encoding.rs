//! GSM 7-bit and UCS2 Encoding

use crate::errors::EncodingError;

/// GSM 7-bit default alphabet
const GSM7_BASIC: &[char] = &[
    '@', '£', '$', '¥', 'è', 'é', 'ù', 'ì', 'ò', 'Ç', '\n', 'Ø', 'ø', '\r', 'Å', 'å',
    'Δ', '_', 'Φ', 'Γ', 'Λ', 'Ω', 'Π', 'Ψ', 'Σ', 'Θ', 'Ξ', '\x1b', 'Æ', 'æ', 'ß', 'É',
    ' ', '!', '"', '#', '¤', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
    '¡', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'Ä', 'Ö', 'Ñ', 'Ü', '§',
    '¿', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'ä', 'ö', 'ñ', 'ü', 'à',
];

/// GSM 7-bit extension table (after ESC)
const GSM7_EXTENSION: &[(u8, char)] = &[
    (0x0A, '\x0C'), // Form feed
    (0x14, '^'),
    (0x28, '{'),
    (0x29, '}'),
    (0x2F, '\\'),
    (0x3C, '['),
    (0x3D, '~'),
    (0x3E, ']'),
    (0x40, '|'),
    (0x65, '€'),
];

/// Encode USSD string
pub fn encode_ussd_string(text: &str, dcs: u8) -> Result<Vec<u8>, EncodingError> {
    match dcs {
        0x0F | 0x00 => encode_gsm7(text),
        0x48 | 0x08 => encode_ucs2(text),
        0x44 | 0x04 => Ok(text.as_bytes().to_vec()),
        _ => Err(EncodingError::UnsupportedDcs(dcs)),
    }
}

/// Decode USSD string
pub fn decode_ussd_string(data: &[u8], dcs: u8) -> Result<String, EncodingError> {
    match dcs {
        0x0F | 0x00 => decode_gsm7(data),
        0x48 | 0x08 => decode_ucs2(data),
        0x44 | 0x04 => String::from_utf8(data.to_vec()).map_err(EncodingError::from),
        _ => Err(EncodingError::UnsupportedDcs(dcs)),
    }
}

/// Encode to GSM 7-bit
pub fn encode_gsm7(text: &str) -> Result<Vec<u8>, EncodingError> {
    let mut septets = Vec::new();
    
    for ch in text.chars() {
        // Check basic alphabet
        if let Some(pos) = GSM7_BASIC.iter().position(|&c| c == ch) {
            septets.push(pos as u8);
        }
        // Check extension table
        else if let Some(&(code, _)) = GSM7_EXTENSION.iter().find(|&&(_, c)| c == ch) {
            septets.push(0x1B); // ESC
            septets.push(code);
        }
        else {
            return Err(EncodingError::InvalidGsm7Char(ch));
        }
    }
    
    // Pack septets into octets
    pack_gsm7(&septets)
}

/// Decode from GSM 7-bit
pub fn decode_gsm7(data: &[u8]) -> Result<String, EncodingError> {
    let septets = unpack_gsm7(data);
    let mut result = String::new();
    let mut escape = false;
    
    for &septet in &septets {
        if escape {
            escape = false;
            if let Some(&(_, ch)) = GSM7_EXTENSION.iter().find(|&&(c, _)| c == septet) {
                result.push(ch);
            }
        } else if septet == 0x1B {
            escape = true;
        } else if (septet as usize) < GSM7_BASIC.len() {
            result.push(GSM7_BASIC[septet as usize]);
        }
    }
    
    Ok(result)
}

/// Pack 7-bit values into octets
fn pack_gsm7(septets: &[u8]) -> Result<Vec<u8>, EncodingError> {
    let mut result = Vec::new();
    let mut bits_pending = 0u16;
    let mut pending_bits = 0u8;
    
    for &septet in septets {
        bits_pending |= (septet as u16) << pending_bits;
        pending_bits += 7;
        
        while pending_bits >= 8 {
            result.push((bits_pending & 0xFF) as u8);
            bits_pending >>= 8;
            pending_bits -= 8;
        }
    }
    
    if pending_bits > 0 {
        result.push((bits_pending & 0xFF) as u8);
    }
    
    Ok(result)
}

/// Unpack octets into 7-bit values
fn unpack_gsm7(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut bits_pending = 0u16;
    let mut pending_bits = 0u8;
    
    for &byte in data {
        bits_pending |= (byte as u16) << pending_bits;
        pending_bits += 8;
        
        while pending_bits >= 7 {
            result.push((bits_pending & 0x7F) as u8);
            bits_pending >>= 7;
            pending_bits -= 7;
        }
    }
    
    result
}

/// Encode to UCS2
pub fn encode_ucs2(text: &str) -> Result<Vec<u8>, EncodingError> {
    let mut result = Vec::with_capacity(text.len() * 2);
    
    for ch in text.chars() {
        let code = ch as u32;
        if code <= 0xFFFF {
            result.push((code >> 8) as u8);
            result.push((code & 0xFF) as u8);
        } else {
            // Surrogate pair for chars > 0xFFFF
            let code = code - 0x10000;
            let high = (0xD800 + (code >> 10)) as u16;
            let low = (0xDC00 + (code & 0x3FF)) as u16;
            result.push((high >> 8) as u8);
            result.push((high & 0xFF) as u8);
            result.push((low >> 8) as u8);
            result.push((low & 0xFF) as u8);
        }
    }
    
    Ok(result)
}

/// Decode from UCS2
pub fn decode_ucs2(data: &[u8]) -> Result<String, EncodingError> {
    if data.len() % 2 != 0 {
        return Err(EncodingError::BufferTooShort);
    }
    
    let mut result = String::new();
    let mut i = 0;
    
    while i < data.len() {
        let code = ((data[i] as u16) << 8) | (data[i + 1] as u16);
        i += 2;
        
        // Check for surrogate pair
        if code >= 0xD800 && code <= 0xDBFF && i + 1 < data.len() {
            let low = ((data[i] as u16) << 8) | (data[i + 1] as u16);
            if low >= 0xDC00 && low <= 0xDFFF {
                i += 2;
                let code_point = 0x10000 + (((code - 0xD800) as u32) << 10) + ((low - 0xDC00) as u32);
                if let Some(ch) = char::from_u32(code_point) {
                    result.push(ch);
                }
                continue;
            }
        }
        
        if let Some(ch) = char::from_u32(code as u32) {
            result.push(ch);
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gsm7_encode_decode() {
        let text = "Hello World!";
        let encoded = encode_gsm7(text).unwrap();
        let decoded = decode_gsm7(&encoded).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_ucs2_encode_decode() {
        let text = "Hello 世界!";
        let encoded = encode_ucs2(text).unwrap();
        let decoded = decode_ucs2(&encoded).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_ussd_string_gsm7() {
        let text = "Balance: 100 NGN";
        let encoded = encode_ussd_string(text, 0x0F).unwrap();
        let decoded = decode_ussd_string(&encoded, 0x0F).unwrap();
        assert_eq!(decoded, text);
    }
}
