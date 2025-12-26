//! M3UA ASP (Application Server Process) State Machine

use super::messages::{M3uaMessage, ProtocolData};
use super::codec;
use crate::config::SigtranConfig;
use crate::errors::M3uaError;
use crate::sctp::{SctpAssociation, StreamConfig};
use crate::types::TrafficModeType;
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, warn};

/// ASP State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspState {
    Down,
    Inactive,
    Active,
}

/// M3UA Endpoint
pub struct M3uaEndpoint {
    /// SCTP association
    sctp: Arc<RwLock<SctpAssociation>>,
    /// Current ASP state
    state: Arc<RwLock<AspState>>,
    /// Local point code
    point_code: u32,
    /// Network indicator
    network_indicator: u8,
    /// Routing contexts
    routing_contexts: Vec<u32>,
    /// Network appearance
    network_appearance: Option<u32>,
    /// Operation timeout
    timeout: Duration,
}

impl M3uaEndpoint {
    /// Create new M3UA endpoint
    #[instrument(skip(config))]
    pub async fn new(config: &SigtranConfig) -> Result<Self, M3uaError> {
        let local_addr: SocketAddr = format!("{}:{}", config.sctp.local_address, config.sctp.port)
            .parse()
            .map_err(|e| M3uaError::AspStateError(format!("Invalid local address: {}", e)))?;
        
        let remote_addr: SocketAddr = format!("{}:{}", config.sctp.remote_address, config.sctp.port)
            .parse()
            .map_err(|e| M3uaError::AspStateError(format!("Invalid remote address: {}", e)))?;

        let stream_config = StreamConfig {
            inbound_streams: config.sctp.streams,
            outbound_streams: config.sctp.streams,
        };

        info!("Connecting M3UA to {}", remote_addr);
        
        let sctp = SctpAssociation::connect(local_addr, remote_addr, stream_config).await?;

        Ok(Self {
            sctp: Arc::new(RwLock::new(sctp)),
            state: Arc::new(RwLock::new(AspState::Down)),
            point_code: config.m3ua.point_code,
            network_indicator: config.m3ua.network_indicator,
            routing_contexts: config.m3ua.routing_contexts.clone().unwrap_or_default(),
            network_appearance: config.m3ua.network_appearance,
            timeout: Duration::from_millis(config.map.operation_timeout_ms),
        })
    }

    /// Get current state
    pub async fn state(&self) -> AspState {
        *self.state.read().await
    }

    /// Bring ASP Up
    #[instrument(skip(self))]
    pub async fn asp_up(&self) -> Result<(), M3uaError> {
        let current_state = *self.state.read().await;
        if current_state != AspState::Down {
            return Err(M3uaError::AspStateError(
                format!("Cannot ASP UP from state {:?}", current_state)
            ));
        }

        info!("Sending ASP UP");
        
        let msg = M3uaMessage::AspUp {
            asp_identifier: None,
            info_string: Some("brivas-sigtran".to_string()),
        };
        
        self.send_message(&msg).await?;

        // Wait for ASP UP ACK
        let response = timeout(self.timeout, self.recv_message()).await
            .map_err(|_| M3uaError::AspStateError("ASP UP ACK timeout".to_string()))??;

        match response {
            M3uaMessage::AspUpAck { .. } => {
                info!("Received ASP UP ACK");
                *self.state.write().await = AspState::Inactive;
                Ok(())
            }
            M3uaMessage::Error { error_code, .. } => {
                Err(M3uaError::ProtocolError(error_code))
            }
            _ => Err(M3uaError::InvalidMessage("Expected ASP UP ACK".to_string())),
        }
    }

    /// Activate ASP
    #[instrument(skip(self))]
    pub async fn asp_active(&self, routing_context: Option<Vec<u32>>) -> Result<(), M3uaError> {
        let current_state = *self.state.read().await;
        if current_state != AspState::Inactive {
            return Err(M3uaError::AspStateError(
                format!("Cannot ASP ACTIVE from state {:?}", current_state)
            ));
        }

        info!("Sending ASP ACTIVE");
        
        let msg = M3uaMessage::AspActive {
            traffic_mode_type: Some(TrafficModeType::Override),
            routing_context,
            info_string: None,
        };
        
        self.send_message(&msg).await?;

        // Wait for ASP ACTIVE ACK
        let response = timeout(self.timeout, self.recv_message()).await
            .map_err(|_| M3uaError::AspStateError("ASP ACTIVE ACK timeout".to_string()))??;

        match response {
            M3uaMessage::AspActiveAck { .. } => {
                info!("Received ASP ACTIVE ACK");
                *self.state.write().await = AspState::Active;
                Ok(())
            }
            M3uaMessage::Error { error_code, .. } => {
                Err(M3uaError::ProtocolError(error_code))
            }
            _ => Err(M3uaError::InvalidMessage("Expected ASP ACTIVE ACK".to_string())),
        }
    }

    /// Bring ASP Down
    #[instrument(skip(self))]
    pub async fn asp_down(&self) -> Result<(), M3uaError> {
        info!("Sending ASP DOWN");
        
        let msg = M3uaMessage::AspDown {
            info_string: None,
        };
        
        self.send_message(&msg).await?;

        // Wait for ASP DOWN ACK
        let response = timeout(self.timeout, self.recv_message()).await
            .map_err(|_| M3uaError::AspStateError("ASP DOWN ACK timeout".to_string()))??;

        match response {
            M3uaMessage::AspDownAck { .. } => {
                info!("Received ASP DOWN ACK");
                *self.state.write().await = AspState::Down;
                Ok(())
            }
            _ => {
                *self.state.write().await = AspState::Down;
                Ok(())
            }
        }
    }

    /// Send MTP3 user data (SCCP)
    #[instrument(skip(self, data), fields(dpc, opc, len = data.len()))]
    pub async fn send_data(
        &self,
        dpc: u32,
        opc: u32,
        si: u8,
        data: &[u8],
    ) -> Result<(), M3uaError> {
        let current_state = *self.state.read().await;
        if current_state != AspState::Active {
            return Err(M3uaError::AspStateError(
                format!("Cannot send data in state {:?}", current_state)
            ));
        }

        let protocol_data = ProtocolData {
            opc,
            dpc,
            si,
            ni: self.network_indicator,
            mp: 0,
            sls: 0,
            data: Bytes::copy_from_slice(data),
        };

        let msg = M3uaMessage::Data {
            network_appearance: self.network_appearance,
            routing_context: self.routing_contexts.first().copied(),
            protocol_data,
            correlation_id: None,
        };

        self.send_message(&msg).await
    }

    /// Receive MTP3 user data
    #[instrument(skip(self))]
    pub async fn recv_data(&self) -> Result<ProtocolData, M3uaError> {
        loop {
            let msg = self.recv_message().await?;
            
            match msg {
                M3uaMessage::Data { protocol_data, .. } => {
                    debug!(
                        "Received data: OPC={}, DPC={}, SI={}, len={}",
                        protocol_data.opc, protocol_data.dpc, 
                        protocol_data.si, protocol_data.data.len()
                    );
                    return Ok(protocol_data);
                }
                M3uaMessage::Heartbeat { data } => {
                    // Respond to heartbeat
                    let ack = M3uaMessage::HeartbeatAck { data };
                    let _ = self.send_message(&ack).await;
                }
                M3uaMessage::Notify { status_type, status_info, .. } => {
                    debug!("Received NOTIFY: type={}, info={}", status_type, status_info);
                }
                _ => {
                    debug!("Ignoring message: {:?}", msg);
                }
            }
        }
    }

    /// Send M3UA message
    async fn send_message(&self, msg: &M3uaMessage) -> Result<(), M3uaError> {
        let encoded = codec::encode_message(msg);
        let sctp = self.sctp.read().await;
        sctp.send(0, &encoded, true).await?;
        Ok(())
    }

    /// Receive M3UA message
    async fn recv_message(&self) -> Result<M3uaMessage, M3uaError> {
        let sctp = self.sctp.read().await;
        let (_, data) = sctp.recv().await?;
        codec::decode_message(&data)
    }

    /// Get local point code
    pub fn point_code(&self) -> u32 {
        self.point_code
    }
}
