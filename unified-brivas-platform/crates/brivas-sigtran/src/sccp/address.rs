//! SCCP Addressing

use crate::errors::SccpError;
use crate::types::{NumberingPlan, NatureOfAddress, EncodingScheme};
use bytes::{Bytes, BytesMut, Buf, BufMut};
use serde::{Deserialize, Serialize};

/// SCCP Address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SccpAddress {
    /// Address indicator
    pub address_indicator: AddressIndicator,
    /// Global Title (optional)
    pub global_title: Option<GlobalTitle>,
    /// Point Code (optional)
    pub point_code: Option<u32>,
    /// Subsystem Number (optional)
    pub subsystem_number: Option<u8>,
}

/// Address Indicator
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AddressIndicator {
    /// National/International indicator
    pub routing_indicator: bool,
    /// Global Title Indicator (0-4)
    pub gti: u8,
    /// SSN Indicator
    pub ssn_indicator: bool,
    /// Point Code Indicator
    pub pc_indicator: bool,
}

impl AddressIndicator {
    pub fn encode(&self) -> u8 {
        let mut ai = 0u8;
        if self.routing_indicator {
            ai |= 0x40;
        }
        ai |= (self.gti & 0x0F) << 2;
        if self.ssn_indicator {
            ai |= 0x02;
        }
        if self.pc_indicator {
            ai |= 0x01;
        }
        ai
    }

    pub fn decode(v: u8) -> Self {
        Self {
            routing_indicator: (v & 0x40) != 0,
            gti: (v >> 2) & 0x0F,
            ssn_indicator: (v & 0x02) != 0,
            pc_indicator: (v & 0x01) != 0,
        }
    }
}

/// Global Title
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlobalTitle {
    /// GTI = 0001: Nature of Address only
    Gt0001 {
        nature_of_address: u8,
        digits: String,
    },
    /// GTI = 0010: Translation Type only
    Gt0010 {
        translation_type: u8,
        digits: String,
    },
    /// GTI = 0011: Translation Type + Numbering Plan + Encoding
    Gt0011 {
        translation_type: u8,
        numbering_plan: u8,
        encoding_scheme: u8,
        digits: String,
    },
    /// GTI = 0100: Full (most common for ITU)
    Gt0100 {
        translation_type: u8,
        numbering_plan: u8,
        encoding_scheme: u8,
        nature_of_address: u8,
        digits: String,
    },
}

impl GlobalTitle {
    /// Create E.164 Global Title (most common)
    pub fn e164(digits: &str) -> Self {
        Self::Gt0100 {
            translation_type: 0,
            numbering_plan: 1,  // E.164
            encoding_scheme: if digits.len() % 2 == 0 { 2 } else { 1 }, // BCD
            nature_of_address: 4, // International
            digits: digits.to_string(),
        }
    }

    /// Get GTI value
    pub fn gti(&self) -> u8 {
        match self {
            Self::Gt0001 { .. } => 1,
            Self::Gt0010 { .. } => 2,
            Self::Gt0011 { .. } => 3,
            Self::Gt0100 { .. } => 4,
        }
    }

    /// Get digits
    pub fn digits(&self) -> &str {
        match self {
            Self::Gt0001 { digits, .. } |
            Self::Gt0010 { digits, .. } |
            Self::Gt0011 { digits, .. } |
            Self::Gt0100 { digits, .. } => digits,
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        let bcd_digits = encode_bcd_digits(self.digits());

        match self {
            Self::Gt0001 { nature_of_address, .. } => {
                buf.put_u8(*nature_of_address & 0x7F);
                buf.put_slice(&bcd_digits);
            }
            Self::Gt0010 { translation_type, .. } => {
                buf.put_u8(*translation_type);
                buf.put_slice(&bcd_digits);
            }
            Self::Gt0011 { translation_type, numbering_plan, encoding_scheme, .. } => {
                buf.put_u8(*translation_type);
                buf.put_u8((*encoding_scheme & 0x0F) | ((*numbering_plan & 0x0F) << 4));
                buf.put_slice(&bcd_digits);
            }
            Self::Gt0100 { translation_type, numbering_plan, encoding_scheme, nature_of_address, .. } => {
                buf.put_u8(*translation_type);
                buf.put_u8((*encoding_scheme & 0x0F) | ((*numbering_plan & 0x0F) << 4));
                buf.put_u8(*nature_of_address & 0x7F);
                buf.put_slice(&bcd_digits);
            }
        }

        buf
    }

    /// Decode from bytes
    pub fn decode(gti: u8, mut data: Bytes) -> Option<Self> {
        match gti {
            1 => {
                if data.remaining() < 1 {
                    return None;
                }
                let noa = data.get_u8();
                let digits = decode_bcd_digits(&data);
                Some(Self::Gt0001 {
                    nature_of_address: noa & 0x7F,
                    digits,
                })
            }
            2 => {
                if data.remaining() < 1 {
                    return None;
                }
                let tt = data.get_u8();
                let digits = decode_bcd_digits(&data);
                Some(Self::Gt0010 {
                    translation_type: tt,
                    digits,
                })
            }
            3 => {
                if data.remaining() < 2 {
                    return None;
                }
                let tt = data.get_u8();
                let np_es = data.get_u8();
                let digits = decode_bcd_digits(&data);
                Some(Self::Gt0011 {
                    translation_type: tt,
                    numbering_plan: (np_es >> 4) & 0x0F,
                    encoding_scheme: np_es & 0x0F,
                    digits,
                })
            }
            4 => {
                if data.remaining() < 3 {
                    return None;
                }
                let tt = data.get_u8();
                let np_es = data.get_u8();
                let noa = data.get_u8();
                let digits = decode_bcd_digits(&data);
                Some(Self::Gt0100 {
                    translation_type: tt,
                    numbering_plan: (np_es >> 4) & 0x0F,
                    encoding_scheme: np_es & 0x0F,
                    nature_of_address: noa & 0x7F,
                    digits,
                })
            }
            _ => None,
        }
    }
}

impl SccpAddress {
    /// Create address with SSN and PC
    pub fn from_ssn_pc(ssn: u8, pc: u32) -> Self {
        Self {
            address_indicator: AddressIndicator {
                routing_indicator: false, // Route on SSN
                gti: 0,
                ssn_indicator: true,
                pc_indicator: true,
            },
            global_title: None,
            point_code: Some(pc),
            subsystem_number: Some(ssn),
        }
    }

    /// Create address with Global Title
    pub fn from_gt(gt: GlobalTitle, ssn: Option<u8>) -> Self {
        Self {
            address_indicator: AddressIndicator {
                routing_indicator: true, // Route on GT
                gti: gt.gti(),
                ssn_indicator: ssn.is_some(),
                pc_indicator: false,
            },
            global_title: Some(gt),
            point_code: None,
            subsystem_number: ssn,
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.put_u8(self.address_indicator.encode());

        if self.address_indicator.pc_indicator {
            if let Some(pc) = self.point_code {
                // ITU format: 14-bit PC in 2 bytes
                buf.put_u16_le(pc as u16);
            }
        }

        if self.address_indicator.ssn_indicator {
            buf.put_u8(self.subsystem_number.unwrap_or(0));
        }

        if self.address_indicator.gti > 0 {
            if let Some(ref gt) = self.global_title {
                buf.put_slice(&gt.encode());
            }
        }

        buf
    }

    /// Decode from bytes
    pub fn decode(mut data: Bytes) -> Option<Self> {
        if data.remaining() < 1 {
            return None;
        }

        let ai = AddressIndicator::decode(data.get_u8());

        let point_code = if ai.pc_indicator && data.remaining() >= 2 {
            Some(data.get_u16_le() as u32)
        } else {
            None
        };

        let subsystem_number = if ai.ssn_indicator && data.remaining() >= 1 {
            Some(data.get_u8())
        } else {
            None
        };

        let global_title = if ai.gti > 0 {
            GlobalTitle::decode(ai.gti, data)
        } else {
            None
        };

        Some(Self {
            address_indicator: ai,
            global_title,
            point_code,
            subsystem_number,
        })
    }
}

/// Encode digits to BCD
fn encode_bcd_digits(digits: &str) -> Vec<u8> {
    let chars: Vec<u8> = digits.chars()
        .filter_map(|c| c.to_digit(16).map(|d| d as u8))
        .collect();
    
    let mut result = Vec::with_capacity((chars.len() + 1) / 2);
    
    for chunk in chars.chunks(2) {
        let byte = if chunk.len() == 2 {
            chunk[0] | (chunk[1] << 4)
        } else {
            chunk[0] | 0xF0
        };
        result.push(byte);
    }
    
    result
}

/// Decode BCD digits
fn decode_bcd_digits(data: &Bytes) -> String {
    let mut result = String::new();
    
    for &byte in data.iter() {
        let low = byte & 0x0F;
        let high = (byte >> 4) & 0x0F;
        
        if low < 10 {
            result.push(char::from_digit(low as u32, 16).unwrap());
        }
        if high < 10 {
            result.push(char::from_digit(high as u32, 16).unwrap());
        }
    }
    
    result
}
