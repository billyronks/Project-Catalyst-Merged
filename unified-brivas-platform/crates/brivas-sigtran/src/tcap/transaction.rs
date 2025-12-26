//! TCAP Transaction Management

use super::{TcapMessage, DialoguePortion, Component};
use crate::errors::TcapError;
use crate::sccp::{SccpEndpoint, SccpAddress, SccpMessage};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Transaction State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Idle,
    InitiationSent,
    InitiationReceived,
    Active,
}

/// TCAP Transaction
#[derive(Debug)]
pub struct TcapTransaction {
    pub originating_tid: Vec<u8>,
    pub destination_tid: Option<Vec<u8>>,
    pub state: TransactionState,
    pub pending_components: Vec<Component>,
}

/// TCAP Endpoint
pub struct TcapEndpoint {
    /// SCCP endpoint
    sccp: Arc<SccpEndpoint>,
    /// Active transactions
    transactions: Arc<RwLock<HashMap<Vec<u8>, TcapTransaction>>>,
    /// Next transaction ID
    next_tid: AtomicU32,
}

impl TcapEndpoint {
    /// Create new TCAP endpoint
    pub fn new(sccp: Arc<SccpEndpoint>) -> Self {
        Self {
            sccp,
            transactions: Arc::new(RwLock::new(HashMap::new())),
            next_tid: AtomicU32::new(1),
        }
    }

    /// Generate new transaction ID
    fn generate_tid(&self) -> Vec<u8> {
        let tid = self.next_tid.fetch_add(1, Ordering::Relaxed);
        tid.to_be_bytes().to_vec()
    }

    /// Start a new transaction (TC-BEGIN)
    #[instrument(skip(self, components), fields(ac = ?application_context))]
    pub async fn begin(
        &self,
        called_address: &SccpAddress,
        application_context: Vec<u32>,
        components: Vec<Component>,
    ) -> Result<Vec<u8>, TcapError> {
        let tid = self.generate_tid();
        
        info!("Starting transaction with TID {:?}", tid);
        
        let dialogue_portion = DialoguePortion {
            application_context_name: application_context,
            user_information: None,
        };
        
        let msg = TcapMessage::Begin {
            originating_transaction_id: tid.clone(),
            dialogue_portion: Some(dialogue_portion),
            component_portion: components,
        };
        
        // Create transaction
        let transaction = TcapTransaction {
            originating_tid: tid.clone(),
            destination_tid: None,
            state: TransactionState::InitiationSent,
            pending_components: Vec::new(),
        };
        
        self.transactions.write().await.insert(tid.clone(), transaction);
        
        // Send via SCCP
        let encoded = msg.encode();
        self.sccp.send_udt(called_address, &self.sccp.local_address(), &encoded).await?;
        
        Ok(tid)
    }

    /// Continue a transaction (TC-CONTINUE)
    #[instrument(skip(self, components))]
    pub async fn continue_transaction(
        &self,
        tid: &[u8],
        components: Vec<Component>,
    ) -> Result<(), TcapError> {
        let mut txns = self.transactions.write().await;
        let txn = txns.get_mut(tid)
            .ok_or(TcapError::TransactionNotFound(tid.to_vec()))?;
        
        let dtid = txn.destination_tid.clone()
            .ok_or(TcapError::InvalidState("No destination TID".to_string()))?;
        
        let msg = TcapMessage::Continue {
            originating_transaction_id: tid.to_vec(),
            destination_transaction_id: dtid,
            dialogue_portion: None,
            component_portion: components,
        };
        
        txn.state = TransactionState::Active;
        
        // Send via SCCP (need called address from context)
        let encoded = msg.encode();
        // Note: In real impl, we'd store the remote address in the transaction
        
        Ok(())
    }

    /// End a transaction (TC-END)
    #[instrument(skip(self, components))]
    pub async fn end(
        &self,
        tid: &[u8],
        components: Vec<Component>,
    ) -> Result<(), TcapError> {
        let mut txns = self.transactions.write().await;
        let txn = txns.remove(tid)
            .ok_or(TcapError::TransactionNotFound(tid.to_vec()))?;
        
        let dtid = txn.destination_tid
            .ok_or(TcapError::InvalidState("No destination TID".to_string()))?;
        
        let msg = TcapMessage::End {
            destination_transaction_id: dtid,
            dialogue_portion: None,
            component_portion: components,
        };
        
        let encoded = msg.encode();
        info!("Ended transaction {:?}", tid);
        
        Ok(())
    }

    /// Receive and process incoming TCAP message
    #[instrument(skip(self))]
    pub async fn receive(&self) -> Result<TcapMessage, TcapError> {
        let sccp_msg = self.sccp.recv().await?;
        
        let data = match sccp_msg {
            SccpMessage::Udt { data, .. } |
            SccpMessage::Xudt { data, .. } => data,
            _ => return Err(TcapError::Sccp(
                crate::errors::SccpError::InvalidMessage("Expected UDT/XUDT".to_string())
            )),
        };
        
        let tcap_msg = TcapMessage::decode(&data)?;
        
        // Update transaction state based on message
        match &tcap_msg {
            TcapMessage::Begin { originating_transaction_id, .. } => {
                debug!("Received BEGIN, TID: {:?}", originating_transaction_id);
                // Create responding transaction
                let txn = TcapTransaction {
                    originating_tid: self.generate_tid(),
                    destination_tid: Some(originating_transaction_id.clone()),
                    state: TransactionState::InitiationReceived,
                    pending_components: Vec::new(),
                };
                self.transactions.write().await.insert(txn.originating_tid.clone(), txn);
            }
            TcapMessage::Continue { destination_transaction_id, originating_transaction_id, .. } => {
                debug!("Received CONTINUE");
                if let Some(txn) = self.transactions.write().await.get_mut(destination_transaction_id) {
                    txn.destination_tid = Some(originating_transaction_id.clone());
                    txn.state = TransactionState::Active;
                }
            }
            TcapMessage::End { destination_transaction_id, .. } => {
                debug!("Received END");
                self.transactions.write().await.remove(destination_transaction_id);
            }
            TcapMessage::Abort { destination_transaction_id, .. } => {
                debug!("Received ABORT");
                self.transactions.write().await.remove(destination_transaction_id);
            }
        }
        
        Ok(tcap_msg)
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, tid: &[u8]) -> Option<TransactionState> {
        self.transactions.read().await.get(tid).map(|t| t.state)
    }
}
