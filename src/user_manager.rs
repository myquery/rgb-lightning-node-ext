use crate::database::Database;
use crate::error::AppError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UserManager {
    database: Database,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub balances: HashMap<String, i64>,
    pub addresses: Vec<String>,
}

impl UserManager {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Extract user_id from request body or headers
    pub fn extract_user_id(body: &serde_json::Value, headers: &axum::http::HeaderMap) -> Option<String> {
        // Try to get user_id from request body
        if let Some(user_id) = body.get("user_id").and_then(|v| v.as_str()) {
            return Some(user_id.to_string());
        }

        // Try to get user_id from headers
        if let Some(user_id) = headers.get("x-user-id").and_then(|v| v.to_str().ok()) {
            return Some(user_id.to_string());
        }

        None
    }

    /// Get or create user context
    pub async fn get_user_context(&self, user_id: &str) -> Result<UserContext, AppError> {
        // Ensure user wallet exists
        if self.database.get_user_wallet(user_id).await?.is_none() {
            self.create_user_wallet(user_id).await?;
        }

        let balances = self.database.get_user_balances(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let addresses = self.database.get_user_addresses(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(UserContext {
            user_id: user_id.to_string(),
            balances,
            addresses,
        })
    }

    /// Create new user wallet
    async fn create_user_wallet(&self, user_id: &str) -> Result<(), AppError> {
        // Generate encrypted mnemonic for user (simplified)
        let mnemonic_encrypted = format!("encrypted_mnemonic_for_{}", user_id);
        let derivation_path = format!("m/84'/1'/0'"); // Testnet derivation path

        self.database.create_user_wallet(user_id, &mnemonic_encrypted, &derivation_path).await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Initialize with zero balances
        self.database.update_user_balance(user_id, None, 0).await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update user balance
    pub async fn update_user_balance(&self, user_id: &str, asset_id: Option<&str>, amount: i64) -> Result<(), AppError> {
        self.database.update_user_balance(user_id, asset_id, amount).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get user balance
    pub async fn get_user_balance(&self, user_id: &str, asset_id: Option<&str>) -> Result<i64, AppError> {
        self.database.get_user_balance(user_id, asset_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Save user transaction
    pub async fn save_user_transaction(&self, user_id: &str, txid: &str, amount: i64, asset_id: Option<&str>, status: &str) -> Result<(), AppError> {
        self.database.save_user_transaction(user_id, txid, amount, asset_id, status).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Save user address
    pub async fn save_user_address(&self, user_id: &str, address: &str) -> Result<(), AppError> {
        self.database.save_user_address(user_id, address).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get user transactions
    pub async fn get_user_transactions(&self, user_id: &str) -> Result<Vec<crate::database::UserTransaction>, AppError> {
        self.database.get_user_transactions(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Save user channel
    pub async fn save_user_channel(&self, user_id: &str, channel_id: &str, peer_pubkey: &str, capacity_sats: i64, status: &str) -> Result<(), AppError> {
        self.database.save_user_channel(user_id, channel_id, peer_pubkey, capacity_sats, status).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get user channels
    pub async fn get_user_channels(&self, user_id: &str) -> Result<Vec<crate::database::UserChannel>, AppError> {
        self.database.get_user_channels(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Save virtual node ID for user
    pub async fn save_virtual_node_id(&self, user_id: &str, virtual_node_id: &str) -> Result<(), AppError> {
        self.database.save_virtual_node_id(user_id, virtual_node_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Get virtual node ID for user
    pub async fn get_virtual_node_id(&self, user_id: &str) -> Result<Option<String>, AppError> {
        self.database.get_virtual_node_id(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }
}