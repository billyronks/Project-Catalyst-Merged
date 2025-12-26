//! SMPP Server implementation for high-performance message handling

use brivas_core::{MessageId, Priority, Result};
use bytes::BytesMut;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use crate::queue::{MessageQueue, QueuedMessage};
use crate::routing::MessageRouter;

/// High-performance SMPP server
#[derive(Clone)]
pub struct SmppServer {
    bind_address: String,
    max_connections: usize,
    sessions: Arc<DashMap<String, SmppSession>>,
    running: Arc<AtomicBool>,
    metrics: Arc<SmppMetrics>,
}

#[derive(Default)]
struct SmppMetrics {
    connections_total: AtomicU64,
    connections_active: AtomicU64,
    messages_received: AtomicU64,
    messages_sent: AtomicU64,
    errors: AtomicU64,
}

/// SMPP session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Open,
    BindReceiver,
    BindTransmitter,
    BindTransceiver,
    Closed,
}

/// SMPP session
#[derive(Clone)]
pub struct SmppSession {
    pub id: String,
    pub system_id: Option<String>,
    pub state: Arc<RwLock<SessionState>>,
    pub throughput_limit: u32, // messages per second
}

/// SMPP command IDs
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandId {
    BindReceiver = 0x00000001,
    BindReceiverResp = 0x80000001,
    BindTransmitter = 0x00000002,
    BindTransmitterResp = 0x80000002,
    BindTransceiver = 0x00000009,
    BindTransceiverResp = 0x80000009,
    SubmitSm = 0x00000004,
    SubmitSmResp = 0x80000004,
    DeliverSm = 0x00000005,
    DeliverSmResp = 0x80000005,
    EnquireLink = 0x00000015,
    EnquireLinkResp = 0x80000015,
    Unbind = 0x00000006,
    UnbindResp = 0x80000006,
    GenericNack = 0x80000000,
}

impl CommandId {
    fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x00000001 => Some(Self::BindReceiver),
            0x80000001 => Some(Self::BindReceiverResp),
            0x00000002 => Some(Self::BindTransmitter),
            0x80000002 => Some(Self::BindTransmitterResp),
            0x00000009 => Some(Self::BindTransceiver),
            0x80000009 => Some(Self::BindTransceiverResp),
            0x00000004 => Some(Self::SubmitSm),
            0x80000004 => Some(Self::SubmitSmResp),
            0x00000005 => Some(Self::DeliverSm),
            0x80000005 => Some(Self::DeliverSmResp),
            0x00000015 => Some(Self::EnquireLink),
            0x80000015 => Some(Self::EnquireLinkResp),
            0x00000006 => Some(Self::Unbind),
            0x80000006 => Some(Self::UnbindResp),
            0x80000000 => Some(Self::GenericNack),
            _ => None,
        }
    }
}

/// SMPP PDU header
#[derive(Debug, Clone)]
pub struct PduHeader {
    pub command_length: u32,
    pub command_id: CommandId,
    pub command_status: u32,
    pub sequence_number: u32,
}

impl SmppServer {
    pub fn new(bind_address: &str, max_connections: usize) -> Self {
        Self {
            bind_address: bind_address.to_string(),
            max_connections,
            sessions: Arc::new(DashMap::new()),
            running: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(SmppMetrics::default()),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("SMPP server stopping");
    }

    pub async fn run(&self, queue: MessageQueue, router: MessageRouter) -> Result<()> {
        let listener = TcpListener::bind(&self.bind_address).await?;
        self.running.store(true, Ordering::SeqCst);

        info!(address = %self.bind_address, "SMPP server listening");

        while self.running.load(Ordering::SeqCst) {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            if self.sessions.len() >= self.max_connections {
                                warn!("Max connections reached, rejecting {}", addr);
                                continue;
                            }

                            let session_id = uuid::Uuid::new_v4().to_string();
                            let session = SmppSession {
                                id: session_id.clone(),
                                system_id: None,
                                state: Arc::new(RwLock::new(SessionState::Open)),
                                throughput_limit: 1000, // Default 1000 TPS per session
                            };

                            self.sessions.insert(session_id.clone(), session.clone());
                            self.metrics.connections_total.fetch_add(1, Ordering::Relaxed);
                            self.metrics.connections_active.fetch_add(1, Ordering::Relaxed);

                            let sessions = self.sessions.clone();
                            let metrics = self.metrics.clone();
                            let queue_clone = queue.clone();
                            let router_clone = router.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(
                                    socket,
                                    session,
                                    queue_clone,
                                    router_clone,
                                ).await {
                                    debug!("Session {} ended: {}", session_id, e);
                                }
                                sessions.remove(&session_id);
                                metrics.connections_active.fetch_sub(1, Ordering::Relaxed);
                            });
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_connection(
        mut socket: TcpStream,
        session: SmppSession,
        queue: MessageQueue,
        router: MessageRouter,
    ) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(4096);

        loop {
            // Read PDU header (16 bytes minimum)
            let n = socket.read_buf(&mut buffer).await?;
            if n == 0 {
                return Ok(()); // Connection closed
            }

            while buffer.len() >= 16 {
                // Parse header
                let command_length = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

                if buffer.len() < command_length {
                    break; // Wait for more data
                }

                let pdu_data = buffer.split_to(command_length);
                let header = Self::parse_header(&pdu_data)?;

                // Handle PDU
                let response = Self::handle_pdu(
                    &header,
                    &pdu_data[16..],
                    &session,
                    &queue,
                    &router,
                ).await?;

                if let Some(resp_bytes) = response {
                    socket.write_all(&resp_bytes).await?;
                }

                if *session.state.read() == SessionState::Closed {
                    return Ok(());
                }
            }
        }
    }

    fn parse_header(data: &BytesMut) -> Result<PduHeader> {
        if data.len() < 16 {
            return Err(brivas_core::BrivasError::Protocol("Header too short".to_string()).into());
        }

        let command_length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let command_id_raw = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let command_status = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let sequence_number = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);

        let command_id = CommandId::from_u32(command_id_raw)
            .ok_or_else(|| brivas_core::BrivasError::Protocol(format!("Unknown command: {:08x}", command_id_raw)))?;

        Ok(PduHeader {
            command_length,
            command_id,
            command_status,
            sequence_number,
        })
    }

    async fn handle_pdu(
        header: &PduHeader,
        body: &[u8],
        session: &SmppSession,
        queue: &MessageQueue,
        router: &MessageRouter,
    ) -> Result<Option<Vec<u8>>> {
        match header.command_id {
            CommandId::BindTransceiver | CommandId::BindTransmitter | CommandId::BindReceiver => {
                // Parse bind request
                let system_id = Self::extract_cstring(body, 0);
                info!(system_id = %system_id, "Bind request");

                *session.state.write() = SessionState::BindTransceiver;

                // Build response
                let resp_id = match header.command_id {
                    CommandId::BindTransceiver => CommandId::BindTransceiverResp,
                    CommandId::BindTransmitter => CommandId::BindTransmitterResp,
                    CommandId::BindReceiver => CommandId::BindReceiverResp,
                    _ => unreachable!(),
                };

                Ok(Some(Self::build_response(resp_id, 0, header.sequence_number, b"BRIVAS\0")))
            }

            CommandId::SubmitSm => {
                // Parse submit_sm PDU
                let source = Self::extract_cstring(body, 3);
                let dest = Self::extract_cstring(body, 4 + source.len());
                let _message = Self::extract_cstring(body, 8 + source.len() + dest.len());

                // Find best route
                let route = router.find_best_route(&dest, &Default::default()).await?;

                // Generate message ID
                let message_id = MessageId::generate();

                // Enqueue message
                queue.enqueue(QueuedMessage {
                    id: message_id.clone(),
                    sender_id: source.clone(),
                    destination: dest.clone(),
                    content: "message".to_string(),
                    priority: Priority::Normal,
                    account_id: session.system_id.clone().unwrap_or_default(),
                    enqueued_at: chrono::Utc::now(),
                    scheduled_at: None,
                    validity_period_secs: None,
                    callback_url: None,
                    metadata: serde_json::json!({
                        "route": route.map(|r| r.id)
                    }),
                }).await?;

                debug!(
                    message_id = %message_id,
                    source = %source,
                    destination = %dest,
                    "SubmitSM processed"
                );

                // Build response with message ID
                let msg_id_bytes = format!("{}\0", message_id);
                Ok(Some(Self::build_response(
                    CommandId::SubmitSmResp,
                    0,
                    header.sequence_number,
                    msg_id_bytes.as_bytes(),
                )))
            }

            CommandId::EnquireLink => {
                Ok(Some(Self::build_response(
                    CommandId::EnquireLinkResp,
                    0,
                    header.sequence_number,
                    &[],
                )))
            }

            CommandId::Unbind => {
                *session.state.write() = SessionState::Closed;
                Ok(Some(Self::build_response(
                    CommandId::UnbindResp,
                    0,
                    header.sequence_number,
                    &[],
                )))
            }

            _ => {
                warn!(command_id = ?header.command_id, "Unhandled command");
                Ok(None)
            }
        }
    }

    fn extract_cstring(data: &[u8], offset: usize) -> String {
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        String::from_utf8_lossy(&data[offset..end]).to_string()
    }

    fn build_response(command_id: CommandId, status: u32, sequence: u32, body: &[u8]) -> Vec<u8> {
        let length = 16 + body.len() as u32;
        let mut response = Vec::with_capacity(length as usize);

        response.extend_from_slice(&length.to_be_bytes());
        response.extend_from_slice(&(command_id as u32).to_be_bytes());
        response.extend_from_slice(&status.to_be_bytes());
        response.extend_from_slice(&sequence.to_be_bytes());
        response.extend_from_slice(body);

        response
    }
}
