//! E2EE Service

use brivas_im_sdk::encryption::{E2eeSession, KeyBundle, EncryptedMessage, E2eeError};
use dashmap::DashMap;
use uuid::Uuid;

/// Encryption service managing E2EE sessions
pub struct EncryptionService {
    /// Active sessions per (user_id, conversation_id)
    sessions: DashMap<(Uuid, Uuid), E2eeSession>,
    /// Key bundles per user
    key_bundles: DashMap<Uuid, KeyBundle>,
}

impl EncryptionService {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            key_bundles: DashMap::new(),
        }
    }

    /// Register a user's key bundle
    pub fn register_key_bundle(&self, bundle: KeyBundle) {
        self.key_bundles.insert(bundle.user_id, bundle);
    }

    /// Get a user's key bundle
    pub fn get_key_bundle(&self, user_id: &Uuid) -> Option<KeyBundle> {
        self.key_bundles.get(user_id).map(|b| b.clone())
    }

    /// Create or get E2EE session for a conversation
    pub fn get_or_create_session(
        &self,
        user_id: Uuid,
        conversation_id: Uuid,
    ) -> E2eeSession {
        let key = (user_id, conversation_id);
        
        self.sessions
            .entry(key)
            .or_insert_with(|| E2eeSession::new(conversation_id))
            .clone()
    }

    /// Update session after key exchange
    pub fn update_session(&self, user_id: Uuid, session: E2eeSession) {
        let key = (user_id, session.conversation_id);
        self.sessions.insert(key, session);
    }

    /// Encrypt a message (placeholder - actual crypto in SDK)
    pub fn encrypt(
        &self,
        _user_id: Uuid,
        _conversation_id: Uuid,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, E2eeError> {
        // TODO: Implement actual encryption using session
        // For now, base64 encode as placeholder
        Ok(base64::encode(plaintext).into_bytes())
    }

    /// Decrypt a message (placeholder - actual crypto in SDK)
    pub fn decrypt(
        &self,
        _user_id: Uuid,
        _conversation_id: Uuid,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, E2eeError> {
        // TODO: Implement actual decryption using session
        // For now, base64 decode as placeholder
        let decoded = base64::decode(ciphertext)
            .map_err(|_| E2eeError::DecryptionFailed("base64 decode failed".to_string()))?;
        Ok(decoded)
    }
}

impl Default for EncryptionService {
    fn default() -> Self {
        Self::new()
    }
}
