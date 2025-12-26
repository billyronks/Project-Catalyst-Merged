//! Common types used across the SIGTRAN stack

use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Point Code (24-bit for ITU, 24-bit for ANSI)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PointCode(pub u32);

impl PointCode {
    /// Create ITU format point code (14-bit)
    pub fn itu(zone: u8, network: u8, sp: u8) -> Self {
        let pc = ((zone as u32 & 0x07) << 11)
            | ((network as u32 & 0xFF) << 3)
            | (sp as u32 & 0x07);
        Self(pc)
    }

    /// Create ANSI format point code (24-bit)
    pub fn ansi(network: u8, cluster: u8, member: u8) -> Self {
        let pc = ((network as u32) << 16)
            | ((cluster as u32) << 8)
            | (member as u32);
        Self(pc)
    }

    /// Get raw value
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for PointCode {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

/// Network Indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NetworkIndicator {
    International = 0,
    InternationalSpare = 1,
    National = 2,
    NationalSpare = 3,
}

impl From<u8> for NetworkIndicator {
    fn from(v: u8) -> Self {
        match v & 0x03 {
            0 => Self::International,
            1 => Self::InternationalSpare,
            2 => Self::National,
            _ => Self::NationalSpare,
        }
    }
}

/// Traffic Mode Type for M3UA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum TrafficModeType {
    Override = 1,
    Loadshare = 2,
    Broadcast = 3,
}

/// Protocol Class for SCCP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolClass {
    pub class: u8,
    pub return_on_error: bool,
}

impl ProtocolClass {
    pub const CLASS_0: Self = Self { class: 0, return_on_error: false };
    pub const CLASS_1: Self = Self { class: 1, return_on_error: false };
    pub const CLASS_2: Self = Self { class: 2, return_on_error: false };
    pub const CLASS_3: Self = Self { class: 3, return_on_error: false };

    pub fn with_return_on_error(mut self) -> Self {
        self.return_on_error = true;
        self
    }

    pub fn encode(&self) -> u8 {
        (self.class & 0x0F) | if self.return_on_error { 0x80 } else { 0 }
    }

    pub fn decode(v: u8) -> Self {
        Self {
            class: v & 0x0F,
            return_on_error: (v & 0x80) != 0,
        }
    }
}

/// Numbering Plan for Global Titles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NumberingPlan {
    Unknown = 0,
    IsdnTelephony = 1,  // E.164
    Generic = 2,
    Data = 3,           // X.121
    Telex = 4,
    MaritimeMobile = 5,
    LandMobile = 6,
    IsdnMobile = 7,
}

/// Nature of Address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NatureOfAddress {
    Unknown = 0,
    SubscriberNumber = 1,
    Reserved = 2,
    NationalSignificant = 3,
    International = 4,
}

/// Encoding Scheme for digits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EncodingScheme {
    Unknown = 0,
    BcdOdd = 1,
    BcdEven = 2,
}

/// USSD Data Coding Scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataCodingScheme(pub u8);

impl DataCodingScheme {
    /// GSM 7-bit default alphabet
    pub const GSM7: Self = Self(0x0F);
    /// UCS2 (16-bit Unicode)
    pub const UCS2: Self = Self(0x48);
    /// 8-bit data
    pub const DATA_8BIT: Self = Self(0x44);

    pub fn is_gsm7(&self) -> bool {
        self.0 == 0x0F || (self.0 & 0x0C) == 0x00
    }

    pub fn is_ucs2(&self) -> bool {
        self.0 == 0x48 || (self.0 & 0x0C) == 0x08
    }
}

/// SM-RP-DA (Destination Address)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmRpDa {
    Imsi(String),
    Lmsi(Vec<u8>),
    ServiceCentreAddress(String),
    NoSmRpDa,
}

/// SM-RP-OA (Originating Address)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmRpOa {
    Msisdn(String),
    ServiceCentreAddress(String),
    NoSmRpOa,
}

/// SM Delivery Outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmDeliveryOutcome {
    MemoryCapacityExceeded,
    AbsentSubscriber,
    SuccessfulTransfer,
}

/// Routing Info result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    pub imsi: String,
    pub msc_number: String,
    pub lmsi: Option<Vec<u8>>,
}

/// USSD Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UssdResponse {
    pub ussd_string: Vec<u8>,
    pub dcs: u8,
    pub release: bool,
}

/// Segment for SCCP segmentation
#[derive(Debug, Clone)]
pub struct Segmentation {
    pub first: bool,
    pub class: u8,
    pub remaining_segments: u8,
    pub reference: u32,
}
