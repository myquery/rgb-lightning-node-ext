use crate::database::Database;
use crate::error::AppError;
use crate::blockchain_balance::BlockchainBalanceService;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserManager {
    database: Database,
    pub blockchain_balance_service: Option<Arc<BlockchainBalanceService>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub balances: HashMap<String, i64>,
    pub addresses: Vec<String>,
}

impl UserManager {
    pub fn new(database: Database) -> Self {
        Self { 
            database,
            blockchain_balance_service: None,
        }
    }

    pub fn with_blockchain_service(database: Database, blockchain_service: Arc<BlockchainBalanceService>) -> Self {
        Self {
            database,
            blockchain_balance_service: Some(blockchain_service),
        }
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

    /// Get user balance using blockchain-based approach (most accurate)
    pub async fn get_user_balance(&self, user_id: &str, asset_id: Option<&str>) -> Result<i64, AppError> {
        // For BTC, use blockchain-based balance (most accurate)
        if asset_id.is_none() || asset_id == Some("BTC") {
            return self.get_btc_balance_from_blockchain(user_id).await;
        }
        
        // For other assets, use database
        self.database.get_user_balance(user_id, asset_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }
    
    /// Get BTC balance from blockchain using user's Bitcoin addresses
    async fn get_btc_balance_from_blockchain(&self, user_id: &str) -> Result<i64, AppError> {
        // Get user's Bitcoin addresses from database
        let addresses = self.database.get_user_addresses(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        if addresses.is_empty() {
            return Ok(0);
        }
        
        let mut total_balance = 0i64;
        let mut successful_queries = 0;
        
        // Query blockchain for each address
        for address in addresses {
            match self.query_address_balance(&address).await {
                Ok(balance) => {
                    total_balance += balance;
                    successful_queries += 1;
                },
                Err(e) => {
                    tracing::warn!("Failed to get balance for address {}: {}", address, e);
                    // Continue with other addresses instead of failing completely
                }
            }
        }
        
        // If all API calls failed due to rate limiting, return cached database balance
        if successful_queries == 0 {
            tracing::warn!("All blockchain API calls failed for user {}, falling back to database balance", user_id);
            return self.database.get_user_balance(user_id, None).await
                .map_err(|e| AppError::Database(e.to_string()));
        }
        
        Ok(total_balance)
    }
    
    /// Query blockchain API for address balance with rate limiting
    async fn query_address_balance(&self, address: &str) -> Result<i64, AppError> {
        let base_url = std::env::var("BLOCKCHAIN_NETWORK_URL")
            .unwrap_or_else(|_| "https://blockstream.info/testnet/api".to_string());
        let url = format!("{}/address/{}", base_url, address);
        
        let client = reqwest::Client::new();
        
        // Add delay to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| AppError::Generic(format!("HTTP request failed: {}", e)))?;
        
        if response.status() == 429 {
            // Rate limited - wait longer and try once more
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let retry_response = client.get(&url)
                .timeout(std::time::Duration::from_secs(15))
                .send()
                .await
                .map_err(|e| AppError::Generic(format!("Retry HTTP request failed: {}", e)))?;
            
            if !retry_response.status().is_success() {
                return Err(AppError::Generic(format!("API retry returned status: {}", retry_response.status())));
            }
            
            let json: serde_json::Value = retry_response.json().await
                .map_err(|e| AppError::Generic(format!("Failed to parse retry JSON: {}", e)))?;
            
            let funded = json["chain_stats"]["funded_txo_sum"].as_u64().unwrap_or(0) as i64;
            let spent = json["chain_stats"]["spent_txo_sum"].as_u64().unwrap_or(0) as i64;
            
            return Ok(funded - spent);
        }
        
        if !response.status().is_success() {
            return Err(AppError::Generic(format!("API returned status: {}", response.status())));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| AppError::Generic(format!("Failed to parse JSON: {}", e)))?;
        
        // Get funded_txo_sum (total received) minus spent_txo_sum (total spent)
        let funded = json["chain_stats"]["funded_txo_sum"].as_u64().unwrap_or(0) as i64;
        let spent = json["chain_stats"]["spent_txo_sum"].as_u64().unwrap_or(0) as i64;
        
        Ok(funded - spent)
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