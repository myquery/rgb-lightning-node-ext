use bitcoin::secp256k1::PublicKey;
use rgb_lib::ContractId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
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
                    contract_id: rgb_transfer.contract_id,
                    required: rgb_transfer.amount,
                    available: rgb_balance,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BalanceError {
    #[error("Insufficient BTC balance: required {required}, available {available}")]
    InsufficientBtc { required: u64, available: u64 },
    #[error("Insufficient RGB balance for {contract_id}: required {required}, available {available}")]
    InsufficientRgb {
        contract_id: ContractId,
        required: u64,
        available: u64,
    },
    #[error("Database error: {0}")]
    Database(String),
}