//! M3UA (MTP3 User Adaptation Layer)
//!
//! RFC 4666 compliant implementation.

mod messages;
mod asp;
mod codec;

pub use messages::{M3uaMessage, ProtocolData};
pub use asp::{M3uaEndpoint, AspState};

use crate::errors::M3uaError;
use crate::types::{NetworkIndicator, TrafficModeType};

/// M3UA Message Class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageClass {
    Management = 0,
    Transfer = 1,
    Ssnm = 2,      // SS7 Signaling Network Management
    Aspsm = 3,     // ASP State Maintenance
    Asptm = 4,     // ASP Traffic Maintenance
    Rkm = 9,       // Routing Key Management
}

/// M3UA Message Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    // Management (Class 0)
    Error,
    Notify,
    
    // Transfer (Class 1)
    Data,
    
    // SSNM (Class 2)
    Duna,   // Destination Unavailable
    Dava,   // Destination Available
    Daud,   // Destination State Audit
    Scon,   // Signaling Congestion
    Dupu,   // Destination User Part Unavailable
    Drst,   // Destination Restricted
    
    // ASPSM (Class 3)
    AspUp,
    AspDown,
    Heartbeat,
    AspUpAck,
    AspDownAck,
    HeartbeatAck,
    
    // ASPTM (Class 4)
    AspActive,
    AspInactive,
    AspActiveAck,
    AspInactiveAck,
}

impl MessageType {
    pub fn class(&self) -> MessageClass {
        match self {
            Self::Error | Self::Notify => MessageClass::Management,
            Self::Data => MessageClass::Transfer,
            Self::Duna | Self::Dava | Self::Daud | Self::Scon | Self::Dupu | Self::Drst => MessageClass::Ssnm,
            Self::AspUp | Self::AspDown | Self::Heartbeat |
            Self::AspUpAck | Self::AspDownAck | Self::HeartbeatAck => MessageClass::Aspsm,
            Self::AspActive | Self::AspInactive |
            Self::AspActiveAck | Self::AspInactiveAck => MessageClass::Asptm,
        }
    }

    pub fn type_value(&self) -> u8 {
        match self {
            Self::Error => 0,
            Self::Notify => 1,
            Self::Data => 1,
            Self::Duna => 1,
            Self::Dava => 2,
            Self::Daud => 3,
            Self::Scon => 4,
            Self::Dupu => 5,
            Self::Drst => 6,
            Self::AspUp => 1,
            Self::AspDown => 2,
            Self::Heartbeat => 3,
            Self::AspUpAck => 4,
            Self::AspDownAck => 5,
            Self::HeartbeatAck => 6,
            Self::AspActive => 1,
            Self::AspInactive => 2,
            Self::AspActiveAck => 3,
            Self::AspInactiveAck => 4,
        }
    }
}

/// M3UA Parameter Tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ParameterTag {
    InfoString = 0x0004,
    RoutingContext = 0x0006,
    DiagnosticInfo = 0x0007,
    HeartbeatData = 0x0009,
    TrafficModeType = 0x000B,
    ErrorCode = 0x000C,
    Status = 0x000D,
    AspIdentifier = 0x0011,
    AffectedPointCode = 0x0012,
    CorrelationId = 0x0013,
    NetworkAppearance = 0x0200,
    ProtocolData = 0x0210,
}
