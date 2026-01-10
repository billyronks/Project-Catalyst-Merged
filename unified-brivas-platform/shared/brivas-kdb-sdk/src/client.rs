//! kdb+ IPC Client

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::{KdbError, Result};

/// kdb+ IPC client
pub struct KdbClient {
    host: String,
    port: u16,
    stream: Arc<Mutex<Option<TcpStream>>>,
}

impl KdbClient {
    /// Create a new kdb+ client
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            stream: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to kdb+ server
    pub async fn connect(&self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e| KdbError::Connection(format!("Invalid address: {}", e)))?;

        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| KdbError::Connection(e.to_string()))?;

        // Send capability byte (version 3, no compression)
        let mut stream = stream;
        stream
            .write_all(b"\x03\x00\x00\x00")
            .await
            .map_err(|e| KdbError::Connection(e.to_string()))?;

        // Read server response
        let mut buf = [0u8; 1];
        stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| KdbError::Connection(e.to_string()))?;

        let mut guard = self.stream.lock().await;
        *guard = Some(stream);

        tracing::info!("Connected to kdb+ at {}:{}", self.host, self.port);
        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let guard = self.stream.lock().await;
        guard.is_some()
    }

    /// Execute a q expression synchronously
    pub async fn execute(&self, query: &str) -> Result<Vec<u8>> {
        let mut guard = self.stream.lock().await;
        let stream = guard
            .as_mut()
            .ok_or_else(|| KdbError::NotConnected)?;

        // Build message: sync + compressed + query
        let query_bytes = query.as_bytes();
        let msg_len = 8 + query_bytes.len() + 1; // header + query + null terminator

        let mut msg = Vec::with_capacity(msg_len);
        msg.push(0x01); // Little endian
        msg.push(0x01); // Message type: sync request
        msg.push(0x00); // Reserved
        msg.push(0x00); // Reserved
        msg.extend(&(msg_len as u32).to_le_bytes()); // Message length
        msg.extend(query_bytes);
        msg.push(0x00); // Null terminator

        stream
            .write_all(&msg)
            .await
            .map_err(|e| KdbError::IO(e.to_string()))?;

        // Read response header
        let mut header = [0u8; 8];
        stream
            .read_exact(&mut header)
            .await
            .map_err(|e| KdbError::IO(e.to_string()))?;

        let response_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;

        // Read response body
        let mut body = vec![0u8; response_len - 8];
        stream
            .read_exact(&mut body)
            .await
            .map_err(|e| KdbError::IO(e.to_string()))?;

        Ok(body)
    }

    /// Execute an async q expression (fire and forget)
    pub async fn execute_async(&self, query: &str) -> Result<()> {
        let mut guard = self.stream.lock().await;
        let stream = guard
            .as_mut()
            .ok_or_else(|| KdbError::NotConnected)?;

        let query_bytes = query.as_bytes();
        let msg_len = 8 + query_bytes.len() + 1;

        let mut msg = Vec::with_capacity(msg_len);
        msg.push(0x01); // Little endian
        msg.push(0x00); // Message type: async request
        msg.push(0x00);
        msg.push(0x00);
        msg.extend(&(msg_len as u32).to_le_bytes());
        msg.extend(query_bytes);
        msg.push(0x00);

        stream
            .write_all(&msg)
            .await
            .map_err(|e| KdbError::IO(e.to_string()))?;

        Ok(())
    }

    /// Close the connection
    pub async fn close(&self) {
        let mut guard = self.stream.lock().await;
        *guard = None;
    }
}
