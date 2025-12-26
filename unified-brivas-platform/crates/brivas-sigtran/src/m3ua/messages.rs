//! M3UA Message Types

use bytes::{Bytes, BytesMut, Buf, BufMut};
use crate::types::TrafficModeType;

/// M3UA Message
#[derive(Debug, Clone)]
pub enum M3uaMessage {
    // ASP State Maintenance (ASPSM)
    AspUp {
        asp_identifier: Option<Vec<u8>>,
        info_string: Option<String>,
    },
    AspUpAck {
        info_string: Option<String>,
    },
    AspDown {
        info_string: Option<String>,
    },
    AspDownAck {
        info_string: Option<String>,
    },
    Heartbeat {
        data: Vec<u8>,
    },
    HeartbeatAck {
        data: Vec<u8>,
    },

    // ASP Traffic Maintenance (ASPTM)
    AspActive {
        traffic_mode_type: Option<TrafficModeType>,
        routing_context: Option<Vec<u32>>,
        info_string: Option<String>,
    },
    AspActiveAck {
        traffic_mode_type: Option<TrafficModeType>,
        routing_context: Option<Vec<u32>>,
        info_string: Option<String>,
    },
    AspInactive {
        routing_context: Option<Vec<u32>>,
        info_string: Option<String>,
    },
    AspInactiveAck {
        routing_context: Option<Vec<u32>>,
        info_string: Option<String>,
    },

    // Transfer
    Data {
        network_appearance: Option<u32>,
        routing_context: Option<u32>,
        protocol_data: ProtocolData,
        correlation_id: Option<u32>,
    },

    // Management
    Error {
        error_code: u32,
        routing_context: Option<Vec<u32>>,
        network_appearance: Option<u32>,
        affected_point_code: Option<Vec<u32>>,
        diagnostic_info: Option<Vec<u8>>,
    },
    Notify {
        status_type: u16,
        status_info: u16,
        asp_identifier: Option<Vec<u8>>,
        routing_context: Option<Vec<u32>>,
        info_string: Option<String>,
    },

    // SSNM
    Duna {
        network_appearance: Option<u32>,
        routing_context: Option<Vec<u32>>,
        affected_point_code: Vec<u32>,
        info_string: Option<String>,
    },
    Dava {
        network_appearance: Option<u32>,
        routing_context: Option<Vec<u32>>,
        affected_point_code: Vec<u32>,
        info_string: Option<String>,
    },
}

/// Protocol Data Unit (MTP3 user data)
#[derive(Debug, Clone)]
pub struct ProtocolData {
    /// Originating Point Code
    pub opc: u32,
    /// Destination Point Code
    pub dpc: u32,
    /// Service Indicator (SCCP = 3)
    pub si: u8,
    /// Network Indicator
    pub ni: u8,
    /// Message Priority
    pub mp: u8,
    /// Signaling Link Selection
    pub sls: u8,
    /// User data (SCCP message)
    pub data: Bytes,
}

impl ProtocolData {
    /// Create new protocol data for SCCP
    pub fn sccp(opc: u32, dpc: u32, ni: u8, data: Bytes) -> Self {
        Self {
            opc,
            dpc,
            si: 3, // SCCP
            ni,
            mp: 0,
            sls: 0,
            data,
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(12 + self.data.len());
        buf.put_u32(self.opc);
        buf.put_u32(self.dpc);
        buf.put_u8(self.si);
        buf.put_u8(self.ni);
        buf.put_u8(self.mp);
        buf.put_u8(self.sls);
        buf.put_slice(&self.data);
        buf
    }

    /// Decode from bytes
    pub fn decode(mut data: Bytes) -> Option<Self> {
        if data.remaining() < 12 {
            return None;
        }

        Some(Self {
            opc: data.get_u32(),
            dpc: data.get_u32(),
            si: data.get_u8(),
            ni: data.get_u8(),
            mp: data.get_u8(),
            sls: data.get_u8(),
            data,
        })
    }
}

impl M3uaMessage {
    /// Get message class
    pub fn class(&self) -> u8 {
        match self {
            Self::Error { .. } | Self::Notify { .. } => 0,
            Self::Data { .. } => 1,
            Self::Duna { .. } | Self::Dava { .. } => 2,
            Self::AspUp { .. } | Self::AspUpAck { .. } |
            Self::AspDown { .. } | Self::AspDownAck { .. } |
            Self::Heartbeat { .. } | Self::HeartbeatAck { .. } => 3,
            Self::AspActive { .. } | Self::AspActiveAck { .. } |
            Self::AspInactive { .. } | Self::AspInactiveAck { .. } => 4,
        }
    }

    /// Get message type
    pub fn message_type(&self) -> u8 {
        match self {
            Self::Error { .. } => 0,
            Self::Notify { .. } => 1,
            Self::Data { .. } => 1,
            Self::Duna { .. } => 1,
            Self::Dava { .. } => 2,
            Self::AspUp { .. } => 1,
            Self::AspDown { .. } => 2,
            Self::Heartbeat { .. } => 3,
            Self::AspUpAck { .. } => 4,
            Self::AspDownAck { .. } => 5,
            Self::HeartbeatAck { .. } => 6,
            Self::AspActive { .. } => 1,
            Self::AspInactive { .. } => 2,
            Self::AspActiveAck { .. } => 3,
            Self::AspInactiveAck { .. } => 4,
        }
    }
}
