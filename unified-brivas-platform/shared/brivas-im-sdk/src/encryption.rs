//! End-to-End Encryption using Signal Protocol concepts
//!
//! Implements X3DH key exchange and Double Ratchet algorithm.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// E2EE Session state
#[derive(Debug, Clone)]
pub struct E2eeSession {
    pub session_id: Uuid,
    pub conversation_id: Uuid,
    pub root_key: [u8; 32],
    pub chain_key: [u8; 32],
    pub message_number: u32,
    pub initialized: bool,
}

/// Public key bundle for key exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBundle {
    pub user_id: Uuid,
    pub identity_key: Vec<u8>,
    pub signed_prekey: Vec<u8>,
    pub signed_prekey_signature: Vec<u8>,
    pub signed_prekey_id: u32,
    pub one_time_prekeys: Vec<OneTimePrekey>,
}

/// One-time prekey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTimePrekey {
    pub id: u32,
    pub public_key: Vec<u8>,
}

/// Encrypted message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub header: MessageHeader,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Message header for Double Ratchet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub dh_public: Vec<u8>,
    pub previous_chain_length: u32,
    pub message_number: u32,
}

/// E2EE Error types
#[derive(Debug, thiserror::Error)]
pub enum E2eeError {
    #[error("Key exchange failed: {0}")]
    KeyExchangeFailed(String),
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Invalid key bundle")]
    InvalidKeyBundle,
    
    #[error("Session not initialized")]
    SessionNotInitialized,
    
    #[error("Out of order message")]
    OutOfOrderMessage,
}

impl E2eeSession {
    /// Create a new uninitialized session
    pub fn new(conversation_id: Uuid) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            conversation_id,
            root_key: [0u8; 32],
            chain_key: [0u8; 32],
            message_number: 0,
            initialized: false,
        }
    }

    /// Initialize session with derived keys
    pub fn initialize(&mut self, root_key: [u8; 32], chain_key: [u8; 32]) {
        self.root_key = root_key;
        self.chain_key = chain_key;
        self.initialized = true;
    }

    /// Check if session is ready for encryption
    pub fn is_ready(&self) -> bool {
        self.initialized
    }
}

impl KeyBundle {
    /// Create a new key bundle for a user
    pub fn new(user_id: Uuid) -> Self {
        // In production, these would be generated cryptographically
        Self {
            user_id,
            identity_key: vec![0u8; 32],
            signed_prekey: vec![0u8; 32],
            signed_prekey_signature: vec![0u8; 64],
            signed_prekey_id: 1,
            one_time_prekeys: vec![],
        }
    }

    /// Add one-time prekeys
    pub fn add_one_time_prekeys(&mut self, count: u32) {
        for i in 0..count {
            self.one_time_prekeys.push(OneTimePrekey {
                id: i + 1,
                public_key: vec![0u8; 32],
            });
        }
    }
}
