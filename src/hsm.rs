use bitcoin::secp256k1::{PublicKey, SecretKey, Secp256k1};
use lightning::sign::KeysManager;
use sha2::{Sha256, Digest};
use std::sync::Arc;

/// HSM service for virtual node ID generation
pub struct HsmService {
    master_keys: Arc<KeysManager>,
    secp_ctx: Secp256k1<bitcoin::secp256k1::All>,
}

impl HsmService {
    pub fn new(master_keys: Arc<KeysManager>) -> Self {
        Self {
            master_keys,
            secp_ctx: Secp256k1::new(),
        }
    }

    /// Generate virtual node ID for a user
    pub fn derive_virtual_node_id(&self, user_id: &str) -> PublicKey {
        let virtual_secret = self.derive_virtual_secret_key(user_id);
        PublicKey::from_secret_key(&self.secp_ctx, &virtual_secret)
    }

    /// Derive virtual secret key for user (internal use only)
    fn derive_virtual_secret_key(&self, user_id: &str) -> SecretKey {
        // Get master node secret
        let master_secret = self.master_keys.get_node_secret_key();
        
        // Create deterministic derivation using HMAC-like approach
        let mut hasher = Sha256::new();
        hasher.update(master_secret.secret_bytes());
        hasher.update(b"virtual_node_");
        hasher.update(user_id.as_bytes());
        let derived_bytes = hasher.finalize();
        
        // Ensure valid secret key
        SecretKey::from_slice(&derived_bytes[..32])
            .expect("derived key should be valid")
    }

    /// Get virtual keys manager for a specific user
    pub fn get_virtual_keys_manager(&self, user_id: &str) -> VirtualKeysManager {
        let virtual_secret = self.derive_virtual_secret_key(user_id);
        VirtualKeysManager::new(self.master_keys.clone(), virtual_secret, user_id.to_string())
    }
}

/// Virtual keys manager that wraps the master keys manager
#[derive(Clone)]
pub struct VirtualKeysManager {
    master_keys: Arc<KeysManager>,
    virtual_node_secret: SecretKey,
    user_id: String,
}

impl VirtualKeysManager {
    fn new(master_keys: Arc<KeysManager>, virtual_node_secret: SecretKey, user_id: String) -> Self {
        Self {
            master_keys,
            virtual_node_secret,
            user_id,
        }
    }

    /// Get the virtual node ID for this user
    pub fn get_virtual_node_id(&self) -> PublicKey {
        let secp_ctx = Secp256k1::new();
        PublicKey::from_secret_key(&secp_ctx, &self.virtual_node_secret)
    }

    /// Get the virtual node secret key
    pub fn get_virtual_node_secret(&self) -> SecretKey {
        self.virtual_node_secret
    }

    /// Get user ID
    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    /// Derive channel keys for this virtual node
    pub fn derive_channel_keys(&self, channel_value_satoshis: u64, params: &[u8; 32]) -> SecretKey {
        // Use master keys manager's derivation but with virtual node context
        let mut hasher = Sha256::new();
        hasher.update(self.virtual_node_secret.secret_bytes());
        hasher.update(b"channel_");
        hasher.update(&channel_value_satoshis.to_be_bytes());
        hasher.update(params);
        let derived_bytes = hasher.finalize();
        
        SecretKey::from_slice(&derived_bytes[..32])
            .expect("derived channel key should be valid")
    }
}