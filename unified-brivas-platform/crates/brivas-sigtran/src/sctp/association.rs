//! SCTP Association Management

use super::{ChunkType, StreamConfig, PPID_M3UA};
use crate::errors::SctpError;
use bytes::{Bytes, BytesMut, Buf, BufMut};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn, instrument};

/// SCTP Association State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssociationState {
    Closed,
    CookieWait,
    CookieEchoed,
    Established,
    ShutdownPending,
    ShutdownSent,
    ShutdownReceived,
    ShutdownAckSent,
}

/// SCTP Association
///
/// Manages a single SCTP association with multi-streaming support.
/// Uses TCP as underlying transport (SCTP-over-TCP for environments without kernel SCTP).
pub struct SctpAssociation {
    /// Underlying stream (TCP fallback for environments without kernel SCTP)
    stream: Arc<Mutex<TcpStream>>,
    /// Local address
    local_addr: SocketAddr,
    /// Remote address
    remote_addr: SocketAddr,
    /// Association state
    state: Arc<RwLock<AssociationState>>,
    /// Stream configuration
    stream_config: StreamConfig,
    /// Next TSN (Transmission Sequence Number)
    next_tsn: AtomicU32,
    /// Heartbeat interval
    heartbeat_interval: Duration,
    /// Receive buffer
    recv_buffer: Arc<Mutex<BytesMut>>,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl SctpAssociation {
    /// Connect to remote peer
    #[instrument(skip_all, fields(remote = %remote_addr))]
    pub async fn connect(
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
        stream_config: StreamConfig,
    ) -> Result<Self, SctpError> {
        info!("Connecting SCTP association to {}", remote_addr);
        
        // Use TCP as SCTP fallback
        let stream = TcpStream::connect(remote_addr).await.map_err(|e| {
            error!("Failed to connect: {}", e);
            SctpError::AssociationFailed(e.to_string())
        })?;
        
        let actual_local = stream.local_addr().map_err(|e| {
            SctpError::AssociationFailed(e.to_string())
        })?;
        
        info!("SCTP association established: {} -> {}", actual_local, remote_addr);
        
        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
            local_addr: actual_local,
            remote_addr,
            state: Arc::new(RwLock::new(AssociationState::Established)),
            stream_config,
            next_tsn: AtomicU32::new(1),
            heartbeat_interval: Duration::from_secs(30),
            recv_buffer: Arc::new(Mutex::new(BytesMut::with_capacity(65536))),
            shutdown_tx: None,
        })
    }

    /// Create from existing TCP stream (for server-side)
    pub fn from_stream(
        stream: TcpStream,
        stream_config: StreamConfig,
    ) -> Result<Self, SctpError> {
        let local_addr = stream.local_addr().map_err(|e| {
            SctpError::AssociationFailed(e.to_string())
        })?;
        let remote_addr = stream.peer_addr().map_err(|e| {
            SctpError::AssociationFailed(e.to_string())
        })?;

        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
            local_addr,
            remote_addr,
            state: Arc::new(RwLock::new(AssociationState::Established)),
            stream_config,
            next_tsn: AtomicU32::new(1),
            heartbeat_interval: Duration::from_secs(30),
            recv_buffer: Arc::new(Mutex::new(BytesMut::with_capacity(65536))),
            shutdown_tx: None,
        })
    }

    /// Send data on a stream
    ///
    /// # Arguments
    /// * `stream_id` - Stream identifier (usually 0 for M3UA)
    /// * `data` - Data to send
    /// * `ordered` - Whether to deliver in order
    #[instrument(skip(self, data), fields(stream_id, len = data.len()))]
    pub async fn send(
        &self,
        stream_id: u16,
        data: &[u8],
        _ordered: bool,
    ) -> Result<(), SctpError> {
        let state = *self.state.read().await;
        if state != AssociationState::Established {
            return Err(SctpError::InvalidState {
                expected: "Established".to_string(),
                actual: format!("{:?}", state),
            });
        }

        // Frame the data with length prefix for TCP transport
        // Format: [4-byte length][2-byte stream_id][4-byte PPID][data]
        let frame_len = 2 + 4 + data.len();
        let mut frame = BytesMut::with_capacity(4 + frame_len);
        frame.put_u32(frame_len as u32);
        frame.put_u16(stream_id);
        frame.put_u32(PPID_M3UA);
        frame.put_slice(data);

        let mut stream = self.stream.lock().await;
        stream.write_all(&frame).await.map_err(|e| {
            error!("Send failed: {}", e);
            SctpError::SendFailed(e.to_string())
        })?;

        debug!("Sent {} bytes on stream {}", data.len(), stream_id);
        Ok(())
    }

    /// Receive data
    ///
    /// Returns (stream_id, data)
    #[instrument(skip(self))]
    pub async fn recv(&self) -> Result<(u16, Bytes), SctpError> {
        let state = *self.state.read().await;
        if state != AssociationState::Established {
            return Err(SctpError::InvalidState {
                expected: "Established".to_string(),
                actual: format!("{:?}", state),
            });
        }

        let mut stream = self.stream.lock().await;
        let mut recv_buf = self.recv_buffer.lock().await;

        // Read length prefix
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.map_err(|e| {
            SctpError::ReceiveFailed(e.to_string())
        })?;
        let frame_len = u32::from_be_bytes(len_buf) as usize;

        // Read frame
        recv_buf.resize(frame_len, 0);
        stream.read_exact(&mut recv_buf[..frame_len]).await.map_err(|e| {
            SctpError::ReceiveFailed(e.to_string())
        })?;

        // Parse frame
        let stream_id = (&recv_buf[0..2]).get_u16();
        let _ppid = (&recv_buf[2..6]).get_u32();
        let data = Bytes::copy_from_slice(&recv_buf[6..]);

        debug!("Received {} bytes on stream {}", data.len(), stream_id);
        Ok((stream_id, data))
    }

    /// Start heartbeat mechanism
    pub fn start_heartbeat(&mut self) -> mpsc::Sender<()> {
        let (tx, mut rx) = mpsc::channel::<()>(1);
        let interval_duration = self.heartbeat_interval;
        let stream = Arc::clone(&self.stream);
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            let mut heartbeat_interval = interval(interval_duration);
            
            loop {
                tokio::select! {
                    _ = heartbeat_interval.tick() => {
                        let current_state = *state.read().await;
                        if current_state != AssociationState::Established {
                            break;
                        }
                        
                        // Send heartbeat (empty frame with special marker)
                        let heartbeat = [0u8; 8]; // Length 0 frame
                        let mut s = stream.lock().await;
                        if s.write_all(&heartbeat).await.is_err() {
                            warn!("Heartbeat failed");
                            break;
                        }
                        debug!("Heartbeat sent");
                    }
                    _ = rx.recv() => {
                        debug!("Heartbeat stopped");
                        break;
                    }
                }
            }
        });

        self.shutdown_tx = Some(tx.clone());
        tx
    }

    /// Close the association gracefully
    #[instrument(skip(self))]
    pub async fn close(&mut self) -> Result<(), SctpError> {
        info!("Closing SCTP association");
        
        // Signal heartbeat to stop
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Update state
        *self.state.write().await = AssociationState::ShutdownPending;

        // Shutdown the stream
        let mut stream = self.stream.lock().await;
        stream.shutdown().await.map_err(|e| {
            SctpError::Io(e)
        })?;

        *self.state.write().await = AssociationState::Closed;
        info!("SCTP association closed");
        Ok(())
    }

    /// Get current state
    pub async fn state(&self) -> AssociationState {
        *self.state.read().await
    }

    /// Get local address
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Get remote address
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Check if association is established
    pub async fn is_established(&self) -> bool {
        *self.state.read().await == AssociationState::Established
    }
}

impl Drop for SctpAssociation {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_association_state_default() {
        assert_eq!(AssociationState::Closed as u8, 0);
    }
}
