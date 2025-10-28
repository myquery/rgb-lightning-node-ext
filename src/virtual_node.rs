use crate::hsm_provider::{HsmProvider, VirtualKeysManager};
use crate::user_manager::UserManager;
use bitcoin::secp256k1::PublicKey;
use std::str::FromStr;
use std::sync::Arc;

/// Virtual node manager for multi-user support
pub struct VirtualNodeManager {
    hsm_provider: Arc<dyn HsmProvider>,
    user_manager: Arc<UserManager>,
}

impl VirtualNodeManager {
    pub fn new(hsm_provider: Arc<dyn HsmProvider>, user_manager: Arc<UserManager>) -> Self {
        Self {
            hsm_provider,
            user_manager,
        }
    }

    /// Get or create virtual node for user
    pub async fn get_virtual_node(&self, user_id: &str) -> Result<VirtualKeysManager, crate::hsm_provider::HsmError> {
        let virtual_keys = self.hsm_provider.get_virtual_keys_manager(user_id).await?;
        let virtual_node_id = virtual_keys.get_virtual_node_id();

        // Check if virtual node ID exists in database, if not save it
        if let Ok(None) = self.user_manager.get_virtual_node_id(user_id).await {
            if let Err(e) = self.user_manager.save_virtual_node_id(user_id, &virtual_node_id.to_string()).await {
                tracing::warn!("Failed to save virtual node ID for user {}: {}", user_id, e);
            }
        }

        Ok(virtual_keys)
    }

    /// Get virtual node ID for user
    pub async fn get_virtual_node_id(&self, user_id: &str) -> Result<PublicKey, crate::hsm_provider::HsmError> {
        // Try to get from database first
        if let Ok(Some(node_id_str)) = self.user_manager.get_virtual_node_id(user_id).await {
            if let Ok(node_id) = PublicKey::from_str(&node_id_str) {
                return Ok(node_id);
            }
        }
        
        // Generate and save if not found
        self.hsm_provider.derive_virtual_node_id(user_id).await
    }
}