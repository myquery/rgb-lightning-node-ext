use bitcoin::secp256k1::PublicKey;
use rgb_lib::ContractId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono;
use crate::virtual_htlc::{VirtualSettlement, RgbTransfer};


/// Virtual balance manager for tracking BTC and RGB balances per virtual node
pub struct VirtualBalanceManager {
    btc_balances: Arc<RwLock<HashMap<PublicKey, u64>>>,
    rgb_balances: Arc<RwLock<HashMap<(PublicKey, ContractId), u64>>>,
    database: Arc<crate::database::Database>,
}

impl VirtualBalanceManager {
    pub fn new(database: Arc<crate::database::Database>) -> Self {
        Self {
            btc_balances: Arc::new(RwLock::new(HashMap::new())),
            rgb_balances: Arc::new(RwLock::new(HashMap::new())),
            database,
        }
    }

    /// Apply virtual HTLC settlement to balances
    pub async fn apply_settlement(&self, settlement: &VirtualSettlement) -> Result<(), BalanceError> {
        // Update BTC balances
        self.update_btc_balance(
            &settlement.from_virtual_node,
            -(settlement.btc_settled as i64),
        ).await?;
        
        self.update_btc_balance(
            &settlement.to_virtual_node,
            settlement.btc_settled as i64,
        ).await?;

        // Update RGB balances if present
        if let Some(ref rgb_transfer) = settlement.rgb_settled {
            self.update_rgb_balance(
                &settlement.from_virtual_node,
                rgb_transfer.contract_id,
                -(rgb_transfer.amount as i64),
            ).await?;
            
            self.update_rgb_balance(
                &settlement.to_virtual_node,
                rgb_transfer.contract_id,
                rgb_transfer.amount as i64,
            ).await?;
        }

        tracing::info!(
            "Applied virtual settlement: {} msat BTC + {:?} RGB",
            settlement.btc_settled,
            settlement.rgb_settled
        );

        Ok(())
    }

    /// Update BTC balance for virtual node
    async fn update_btc_balance(&self, virtual_node: &PublicKey, delta: i64) -> Result<(), BalanceError> {
        let mut balances = self.btc_balances.write().await;
        let current = balances.get(virtual_node).copied().unwrap_or(0);
        let new_balance = (current as i64 + delta).max(0) as u64;
        balances.insert(*virtual_node, new_balance);

        // Persist to database
        let user_id = virtual_node.to_string(); // TODO: Map to actual user_id
        self.database
            .update_user_balance(&user_id, None, new_balance as i64)
            .await
            .map_err(|e| BalanceError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update RGB balance for virtual node
    async fn update_rgb_balance(
        &self,
        virtual_node: &PublicKey,
        contract_id: ContractId,
        delta: i64,
    ) -> Result<(), BalanceError> {
        let mut balances = self.rgb_balances.write().await;
        let key = (*virtual_node, contract_id);
        let current = balances.get(&key).copied().unwrap_or(0);
        let new_balance = (current as i64 + delta).max(0) as u64;
        balances.insert(key, new_balance);

        // Persist to database
        let user_id = virtual_node.to_string(); // TODO: Map to actual user_id
        let asset_id = contract_id.to_string();
        self.database
            .update_user_balance(&user_id, Some(&asset_id), new_balance as i64)
            .await
            .map_err(|e| BalanceError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get BTC balance for virtual node
    pub async fn get_btc_balance(&self, virtual_node: &PublicKey) -> u64 {
        let balances = self.btc_balances.read().await;
        balances.get(virtual_node).copied().unwrap_or(0)
    }

    /// Get RGB balance for virtual node
    pub async fn get_rgb_balance(&self, virtual_node: &PublicKey, contract_id: ContractId) -> u64 {
        let balances = self.rgb_balances.read().await;
        let key = (*virtual_node, contract_id);
        balances.get(&key).copied().unwrap_or(0)
    }

    /// Check if virtual node has sufficient balance for transfer
    pub async fn check_sufficient_balance(
        &self,
        virtual_node: &PublicKey,
        btc_amount: u64,
        rgb_transfer: Option<&RgbTransfer>,
    ) -> Result<(), BalanceError> {
        // Check BTC balance
        let btc_balance = self.get_btc_balance(virtual_node).await;
        if btc_balance < btc_amount {
            return Err(BalanceError::InsufficientBtc {
                required: btc_amount,
                available: btc_balance,
            });
        }

        // Check RGB balance if needed
        if let Some(rgb_transfer) = rgb_transfer {
            let rgb_balance = self.get_rgb_balance(virtual_node, rgb_transfer.contract_id).await;
            if rgb_balance < rgb_transfer.amount {
                return Err(BalanceError::InsufficientRgb {
                    contract_id: rgb_transfer.contract_id.to_string(),
                    required: rgb_transfer.amount,
                    available: rgb_balance,
                });
            }
        }

        Ok(())
    }

    /// Execute real Bitcoin transfer between user addresses
    pub async fn execute_virtual_transfer(
        &self,
        from_user_id: i64,
        to_user_id: i64,
        amount_sats: u64,
        unlocked_state: &crate::utils::UnlockedAppState,
        app_state: &crate::utils::AppState,
    ) -> Result<String, BalanceError> {
        tracing::info!("Creating REAL Bitcoin transfer: {} -> {} ({} sats)", from_user_id, to_user_id, amount_sats);
        
        // Get user manager for address lookup
        let user_manager = app_state.user_manager.lock().await;
        let user_mgr = user_manager.as_ref().ok_or_else(|| BalanceError::Database("User manager not available".to_string()))?;
        
        // Get sender's Bitcoin address
        let sender_addresses = self.database.get_user_addresses(&from_user_id.to_string()).await
            .map_err(|e| BalanceError::Database(e.to_string()))?;
        
        if sender_addresses.is_empty() {
            return Err(BalanceError::Database("Sender has no Bitcoin addresses".to_string()));
        }
        
        // Get receiver's Bitcoin address
        let receiver_addresses = self.database.get_user_addresses(&to_user_id.to_string()).await
            .map_err(|e| BalanceError::Database(e.to_string()))?;
        
        if receiver_addresses.is_empty() {
            return Err(BalanceError::Database("Receiver has no Bitcoin addresses".to_string()));
        }
        
        let receiver_address = &receiver_addresses[0]; // Use first address
        tracing::info!("Sending {} sats to receiver address: {}", amount_sats, receiver_address);
        
        // Check sender's actual blockchain balance
        let sender_balance = user_mgr.get_user_balance(&from_user_id.to_string(), None).await
            .map_err(|e| BalanceError::Database(e.to_string()))? as u64;
        
        if sender_balance < amount_sats {
            return Err(BalanceError::InsufficientBtc {
                required: amount_sats,
                available: sender_balance,
            });
        }
        
        // Create real Bitcoin transaction
        let transaction_id = format!("btc_{}_{}_{}_{}", from_user_id, to_user_id, amount_sats, chrono::Utc::now().timestamp());
        
        tracing::info!("Creating real Bitcoin transaction: {} sats to {}", amount_sats, receiver_address);
        
        // Use RGB wallet's send_btc function for real Bitcoin transfer
        let txid = match unlocked_state.rgb_send_btc(
            receiver_address.clone(),
            amount_sats,
            25, // fee_rate (25 sat/vB)
            false, // skip_sync
        ) {
            Ok(txid) => {
                tracing::info!("Real Bitcoin transaction created: {}", txid);
                txid
            }
            Err(e) => {
                tracing::error!("Failed to create Bitcoin transaction: {:?}", e);
                return Err(BalanceError::Database(format!("Bitcoin transaction failed: {:?}", e)));
            }
        };
        
        // Record the real Bitcoin transaction in database for tracking
        tracing::info!("Recording Bitcoin transaction: {}", txid);
        
        match sqlx::query!(
            "INSERT INTO ln_user_transactions (id, user_id, txid, amount, status) VALUES ($1, $2, $3, $4, 'completed')",
            uuid::Uuid::new_v4(),
            from_user_id.to_string(),
            txid,
            -(amount_sats as i64)
        )
        .execute(self.database.pool())
        .await {
            Ok(_) => tracing::info!("Sender transaction recorded"),
            Err(e) => tracing::warn!("Failed to record sender transaction: {}", e),
        }
        
        match sqlx::query!(
            "INSERT INTO ln_user_transactions (id, user_id, txid, amount, status) VALUES ($1, $2, $3, $4, 'completed')",
            uuid::Uuid::new_v4(),
            to_user_id.to_string(),
            txid,
            amount_sats as i64
        )
        .execute(self.database.pool())
        .await {
            Ok(_) => tracing::info!("Receiver transaction recorded"),
            Err(e) => tracing::warn!("Failed to record receiver transaction: {}", e),
        }
        
        tracing::info!("Real Bitcoin transfer completed: {} sats sent from user {} to user {}", amount_sats, from_user_id, to_user_id);
        tracing::info!("Bitcoin transaction ID: {}", txid);
        
        Ok(txid)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BalanceError {
    #[error("Insufficient BTC balance")]
    InsufficientBtc { required: u64, available: u64 },
    #[error("Insufficient RGB balance")]
    InsufficientRgb {
        contract_id: String,
        required: u64,
        available: u64,
    },
    #[error("Database error: {0}")]
    Database(String),
}

impl VirtualBalanceManager {
    /// Get user's actual BTC balance from their real UTXOs (represents real spendable funds)
    async fn get_actual_user_balance(
        &self,
        telegram_id: &str,
        _unlocked_state: &crate::utils::UnlockedAppState,
    ) -> Result<u64, anyhow::Error> {
        let user_telegram_id = telegram_id.parse::<i64>()
            .map_err(|_| anyhow::anyhow!("Invalid telegram_id: {}", telegram_id))?;
        
        // Check user_utxos table for actual spendable balance using telegram_id
        let balance = sqlx::query!(
            "SELECT COALESCE(SUM(amount)::BIGINT, 0) as balance FROM user_utxos WHERE user_id = $1 AND spent = false",
            user_telegram_id
        )
        .fetch_one(self.database.pool())
        .await
        .map(|row| row.balance.unwrap_or(0))
        .unwrap_or(0);
        
        let actual_balance = balance.max(0) as u64;
        
        tracing::info!("User {} (telegram_id) actual UTXO balance: {} sats", telegram_id, actual_balance);
        
        Ok(actual_balance)
    }
}