use crate::hsm_provider::VirtualKeysManager;
use crate::utils::AppState;
use crate::virtual_channel::VirtualChannelManager;
use crate::virtual_htlc::VirtualHtlcManager;
use crate::virtual_balance::VirtualBalanceManager;
use crate::virtual_router::VirtualRouter;
use bitcoin::secp256k1::PublicKey;
use std::sync::Arc;

/// Virtual node context for multi-user operations
#[derive(Clone)]
pub struct VirtualNodeContext {
    pub user_id: String,
    pub virtual_node_id: PublicKey,
    pub virtual_keys: VirtualKeysManager,
    pub channel_manager: Option<VirtualChannelManager>,
}

impl VirtualNodeContext {
    /// Extract virtual node context from request and app state
    pub async fn from_request(
        payload: &serde_json::Value,
        headers: &axum::http::HeaderMap,
        app_state: &Arc<AppState>,
    ) -> Option<Self> {
        // Extract user_id from request
        let user_id = crate::user_manager::UserManager::extract_user_id(payload, headers)?;
        
        // Get virtual node manager
        let virtual_mgr = app_state.virtual_node_manager.lock().await;
        let virtual_mgr = virtual_mgr.as_ref()?;
        
        // Get virtual node for user
        let virtual_keys = virtual_mgr.get_virtual_node(&user_id).await.ok()?;
        let virtual_node_id = virtual_keys.get_virtual_node_id();
        
        // Get virtual channel manager if database is available
        let channel_manager = if let Some(database) = app_state.database.lock().await.as_ref() {
            Some(VirtualChannelManager::new(database.clone()))
        } else {
            None
        };
        
        Some(Self {
            user_id,
            virtual_node_id,
            virtual_keys,
            channel_manager,
        })
    }
    
    /// Check if this virtual node owns a channel
    pub async fn owns_channel(&self, channel_id: &str) -> bool {
        if let Some(ref mgr) = self.channel_manager {
            if let Ok(Some(virtual_node_id)) = mgr.get_virtual_node_for_channel(channel_id).await {
                return virtual_node_id == self.virtual_node_id.to_string();
            }
        }
        false
    }
    
    /// Get channels owned by this virtual node
    pub async fn get_owned_channels(&self) -> Vec<String> {
        if let Some(ref mgr) = self.channel_manager {
            mgr.get_channels_for_virtual_node(&self.virtual_node_id.to_string()).await.unwrap_or_default()
        } else {
            vec![]
        }
    }
    
    /// Get payments for this virtual node
    pub async fn get_owned_payments(&self) -> Vec<String> {
        if let Some(ref mgr) = self.channel_manager {
            mgr.get_payments_for_virtual_node(&self.virtual_node_id.to_string()).await.unwrap_or_default()
        } else {
            vec![]
        }
    }
}