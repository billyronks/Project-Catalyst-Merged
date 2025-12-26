//! M3UA Message Encoding/Decoding

use super::messages::{M3uaMessage, ProtocolData};
use super::ParameterTag;
use crate::errors::M3uaError;
use crate::types::TrafficModeType;
use bytes::{Bytes, BytesMut, Buf, BufMut};

/// M3UA Version
const M3UA_VERSION: u8 = 1;

/// Encode M3UA message
pub fn encode_message(msg: &M3uaMessage) -> BytesMut {
    let mut params = BytesMut::new();
    
    match msg {
        M3uaMessage::AspUp { asp_identifier, info_string } => {
            if let Some(id) = asp_identifier {
                encode_parameter(&mut params, ParameterTag::AspIdentifier, id);
            }
            if let Some(info) = info_string {
                encode_parameter(&mut params, ParameterTag::InfoString, info.as_bytes());
            }
        }
        M3uaMessage::AspUpAck { info_string } |
        M3uaMessage::AspDown { info_string } |
        M3uaMessage::AspDownAck { info_string } => {
            if let Some(info) = info_string {
                encode_parameter(&mut params, ParameterTag::InfoString, info.as_bytes());
            }
        }
        M3uaMessage::AspActive { traffic_mode_type, routing_context, info_string } |
        M3uaMessage::AspActiveAck { traffic_mode_type, routing_context, info_string } => {
            if let Some(mode) = traffic_mode_type {
                let mode_val = match mode {
                    TrafficModeType::Override => 1u32,
                    TrafficModeType::Loadshare => 2,
                    TrafficModeType::Broadcast => 3,
                };
                encode_parameter(&mut params, ParameterTag::TrafficModeType, &mode_val.to_be_bytes());
            }
            if let Some(rc) = routing_context {
                let rc_bytes: Vec<u8> = rc.iter().flat_map(|v| v.to_be_bytes()).collect();
                encode_parameter(&mut params, ParameterTag::RoutingContext, &rc_bytes);
            }
            if let Some(info) = info_string {
                encode_parameter(&mut params, ParameterTag::InfoString, info.as_bytes());
            }
        }
        M3uaMessage::AspInactive { routing_context, info_string } |
        M3uaMessage::AspInactiveAck { routing_context, info_string } => {
            if let Some(rc) = routing_context {
                let rc_bytes: Vec<u8> = rc.iter().flat_map(|v| v.to_be_bytes()).collect();
                encode_parameter(&mut params, ParameterTag::RoutingContext, &rc_bytes);
            }
            if let Some(info) = info_string {
                encode_parameter(&mut params, ParameterTag::InfoString, info.as_bytes());
            }
        }
        M3uaMessage::Heartbeat { data } |
        M3uaMessage::HeartbeatAck { data } => {
            encode_parameter(&mut params, ParameterTag::HeartbeatData, data);
        }
        M3uaMessage::Data { network_appearance, routing_context, protocol_data, correlation_id } => {
            if let Some(na) = network_appearance {
                encode_parameter(&mut params, ParameterTag::NetworkAppearance, &na.to_be_bytes());
            }
            if let Some(rc) = routing_context {
                encode_parameter(&mut params, ParameterTag::RoutingContext, &rc.to_be_bytes());
            }
            
            // Encode Protocol Data
            let pd_encoded = protocol_data.encode();
            encode_parameter(&mut params, ParameterTag::ProtocolData, &pd_encoded);
            
            if let Some(cid) = correlation_id {
                encode_parameter(&mut params, ParameterTag::CorrelationId, &cid.to_be_bytes());
            }
        }
        M3uaMessage::Error { error_code, routing_context, network_appearance, affected_point_code, diagnostic_info } => {
            encode_parameter(&mut params, ParameterTag::ErrorCode, &error_code.to_be_bytes());
            if let Some(rc) = routing_context {
                let rc_bytes: Vec<u8> = rc.iter().flat_map(|v| v.to_be_bytes()).collect();
                encode_parameter(&mut params, ParameterTag::RoutingContext, &rc_bytes);
            }
            if let Some(na) = network_appearance {
                encode_parameter(&mut params, ParameterTag::NetworkAppearance, &na.to_be_bytes());
            }
            if let Some(apc) = affected_point_code {
                let apc_bytes: Vec<u8> = apc.iter().flat_map(|v| v.to_be_bytes()).collect();
                encode_parameter(&mut params, ParameterTag::AffectedPointCode, &apc_bytes);
            }
            if let Some(diag) = diagnostic_info {
                encode_parameter(&mut params, ParameterTag::DiagnosticInfo, diag);
            }
        }
        M3uaMessage::Notify { status_type, status_info, asp_identifier, routing_context, info_string } => {
            let mut status = BytesMut::with_capacity(4);
            status.put_u16(*status_type);
            status.put_u16(*status_info);
            encode_parameter(&mut params, ParameterTag::Status, &status);
            
            if let Some(id) = asp_identifier {
                encode_parameter(&mut params, ParameterTag::AspIdentifier, id);
            }
            if let Some(rc) = routing_context {
                let rc_bytes: Vec<u8> = rc.iter().flat_map(|v| v.to_be_bytes()).collect();
                encode_parameter(&mut params, ParameterTag::RoutingContext, &rc_bytes);
            }
            if let Some(info) = info_string {
                encode_parameter(&mut params, ParameterTag::InfoString, info.as_bytes());
            }
        }
        M3uaMessage::Duna { network_appearance, routing_context, affected_point_code, info_string } |
        M3uaMessage::Dava { network_appearance, routing_context, affected_point_code, info_string } => {
            if let Some(na) = network_appearance {
                encode_parameter(&mut params, ParameterTag::NetworkAppearance, &na.to_be_bytes());
            }
            if let Some(rc) = routing_context {
                let rc_bytes: Vec<u8> = rc.iter().flat_map(|v| v.to_be_bytes()).collect();
                encode_parameter(&mut params, ParameterTag::RoutingContext, &rc_bytes);
            }
            let apc_bytes: Vec<u8> = affected_point_code.iter().flat_map(|v| v.to_be_bytes()).collect();
            encode_parameter(&mut params, ParameterTag::AffectedPointCode, &apc_bytes);
            if let Some(info) = info_string {
                encode_parameter(&mut params, ParameterTag::InfoString, info.as_bytes());
            }
        }
    }

    // Build header
    let length = 8 + params.len() as u32;
    let mut buf = BytesMut::with_capacity(length as usize);
    buf.put_u8(M3UA_VERSION);
    buf.put_u8(0); // Reserved
    buf.put_u8(msg.class());
    buf.put_u8(msg.message_type());
    buf.put_u32(length);
    buf.put_slice(&params);
    
    buf
}

/// Encode a TLV parameter
fn encode_parameter(buf: &mut BytesMut, tag: ParameterTag, value: &[u8]) {
    let length = 4 + value.len() as u16;
    buf.put_u16(tag as u16);
    buf.put_u16(length);
    buf.put_slice(value);
    
    // Pad to 4-byte boundary
    let padding = (4 - (value.len() % 4)) % 4;
    for _ in 0..padding {
        buf.put_u8(0);
    }
}

/// Decode M3UA message
pub fn decode_message(data: &[u8]) -> Result<M3uaMessage, M3uaError> {
    if data.len() < 8 {
        return Err(M3uaError::InvalidMessage("Message too short".to_string()));
    }

    let mut buf = Bytes::copy_from_slice(data);
    
    let version = buf.get_u8();
    if version != M3UA_VERSION {
        return Err(M3uaError::InvalidMessage(format!("Invalid version: {}", version)));
    }
    
    let _reserved = buf.get_u8();
    let msg_class = buf.get_u8();
    let msg_type = buf.get_u8();
    let length = buf.get_u32() as usize;

    if data.len() < length {
        return Err(M3uaError::InvalidMessage("Incomplete message".to_string()));
    }

    // Parse parameters
    let params = parse_parameters(&buf)?;

    // Construct message based on class and type
    match (msg_class, msg_type) {
        (3, 1) => Ok(M3uaMessage::AspUp {
            asp_identifier: params.get(&(ParameterTag::AspIdentifier as u16)).cloned(),
            info_string: get_string_param(&params, ParameterTag::InfoString),
        }),
        (3, 4) => Ok(M3uaMessage::AspUpAck {
            info_string: get_string_param(&params, ParameterTag::InfoString),
        }),
        (3, 2) => Ok(M3uaMessage::AspDown {
            info_string: get_string_param(&params, ParameterTag::InfoString),
        }),
        (3, 5) => Ok(M3uaMessage::AspDownAck {
            info_string: get_string_param(&params, ParameterTag::InfoString),
        }),
        (4, 1) => Ok(M3uaMessage::AspActive {
            traffic_mode_type: get_traffic_mode(&params),
            routing_context: get_routing_context(&params),
            info_string: get_string_param(&params, ParameterTag::InfoString),
        }),
        (4, 3) => Ok(M3uaMessage::AspActiveAck {
            traffic_mode_type: get_traffic_mode(&params),
            routing_context: get_routing_context(&params),
            info_string: get_string_param(&params, ParameterTag::InfoString),
        }),
        (1, 1) => {
            let pd_bytes = params.get(&(ParameterTag::ProtocolData as u16))
                .ok_or_else(|| M3uaError::InvalidMessage("Missing protocol data".to_string()))?;
            let protocol_data = ProtocolData::decode(Bytes::copy_from_slice(pd_bytes))
                .ok_or_else(|| M3uaError::InvalidMessage("Invalid protocol data".to_string()))?;
            
            Ok(M3uaMessage::Data {
                network_appearance: get_u32_param(&params, ParameterTag::NetworkAppearance),
                routing_context: get_u32_param(&params, ParameterTag::RoutingContext),
                protocol_data,
                correlation_id: get_u32_param(&params, ParameterTag::CorrelationId),
            })
        }
        (0, 0) => Ok(M3uaMessage::Error {
            error_code: get_u32_param(&params, ParameterTag::ErrorCode).unwrap_or(0),
            routing_context: get_routing_context(&params),
            network_appearance: get_u32_param(&params, ParameterTag::NetworkAppearance),
            affected_point_code: get_routing_context(&params), // Same format
            diagnostic_info: params.get(&(ParameterTag::DiagnosticInfo as u16)).cloned(),
        }),
        _ => Err(M3uaError::InvalidMessage(format!(
            "Unknown message: class={}, type={}", msg_class, msg_type
        ))),
    }
}

/// Parse TLV parameters
fn parse_parameters(buf: &Bytes) -> Result<std::collections::HashMap<u16, Vec<u8>>, M3uaError> {
    let mut params = std::collections::HashMap::new();
    let mut cursor = buf.clone();
    
    while cursor.remaining() >= 4 {
        let tag = cursor.get_u16();
        let length = cursor.get_u16() as usize;
        
        if length < 4 || cursor.remaining() < length - 4 {
            break;
        }
        
        let value_len = length - 4;
        let value = cursor.copy_to_bytes(value_len).to_vec();
        params.insert(tag, value);
        
        // Skip padding
        let padding = (4 - (value_len % 4)) % 4;
        if cursor.remaining() >= padding {
            cursor.advance(padding);
        }
    }
    
    Ok(params)
}

fn get_string_param(params: &std::collections::HashMap<u16, Vec<u8>>, tag: ParameterTag) -> Option<String> {
    params.get(&(tag as u16))
        .and_then(|v| String::from_utf8(v.clone()).ok())
}

fn get_u32_param(params: &std::collections::HashMap<u16, Vec<u8>>, tag: ParameterTag) -> Option<u32> {
    params.get(&(tag as u16))
        .filter(|v| v.len() >= 4)
        .map(|v| u32::from_be_bytes([v[0], v[1], v[2], v[3]]))
}

fn get_routing_context(params: &std::collections::HashMap<u16, Vec<u8>>) -> Option<Vec<u32>> {
    params.get(&(ParameterTag::RoutingContext as u16))
        .map(|v| {
            v.chunks_exact(4)
                .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
                .collect()
        })
}

fn get_traffic_mode(params: &std::collections::HashMap<u16, Vec<u8>>) -> Option<TrafficModeType> {
    get_u32_param(params, ParameterTag::TrafficModeType).and_then(|v| {
        match v {
            1 => Some(TrafficModeType::Override),
            2 => Some(TrafficModeType::Loadshare),
            3 => Some(TrafficModeType::Broadcast),
            _ => None,
        }
    })
}
