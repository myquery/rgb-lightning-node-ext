use crate::bitcoind::BitcoindClient;
use crate::error::AppError;
use anyhow::Result;
use bitcoin::Address;
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;

/// Blockchain-based balance querying service
#[derive(Clone)]
pub struct BlockchainBalanceService {
    bitcoind_client: Arc<BitcoindClient>,
}

impl BlockchainBalanceService {
    pub fn new(bitcoind_client: Arc<BitcoindClient>) -> Self {
        Self { bitcoind_client }
    }

    /// Get real Bitcoin balance from blockchain using user's address
    pub async fn get_address_balance(&self, address: &str) -> Result<i64, AppError> {
        // Validate Bitcoin address
        let _addr = Address::from_str(address)
            .map_err(|_| AppError::InvalidRequest(format!("Invalid Bitcoin address: {}", address)))?;

        // Get UTXOs for this address from bitcoind
        let utxos = self.get_address_utxos(address).await?;
        
        // Sum up all UTXO values
        let total_balance = utxos.iter()
            .map(|utxo| utxo.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .sum::<f64>();

        // Convert BTC to satoshis
        Ok((total_balance * 100_000_000.0) as i64)
    }

    /// Get all UTXOs for a Bitcoin address
    async fn get_address_utxos(&self, address: &str) -> Result<Vec<Value>, AppError> {
        // Use scantxoutset to find UTXOs for this address
        let scan_objects = vec![format!("addr({})", address)];
        let params = vec![
            serde_json::json!("start"),
            serde_json::json!(scan_objects)
        ];

        let response = match self.bitcoind_client.bitcoind_rpc_client
            .call_method::<Value>("scantxoutset", &params)
            .await {
            Ok(response) => response,
            Err(e) if e.to_string().contains("Scan already in progress") => {
                // Abort existing scan and retry
                tracing::warn!("Scan already in progress for address {}, aborting and retrying...", address);
                let _ = self.bitcoind_client.bitcoind_rpc_client
                    .call_method::<Value>("scantxoutset", &[serde_json::json!("abort")])
                    .await;
                
                // Wait briefly for abort to complete
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                // Retry the scan
                self.bitcoind_client.bitcoind_rpc_client
                    .call_method::<Value>("scantxoutset", &params)
                    .await
                    .map_err(|e| AppError::Generic(format!("Failed to scan UTXO set after retry: {}", e)))?
            }
            Err(e) => return Err(AppError::Generic(format!("Failed to scan UTXO set: {}", e)))
        };

        // Extract UTXOs from response
        let utxos = response.get("unspents")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .clone();

        Ok(utxos)
    }

    /// Get balance for multiple addresses (for users with multiple addresses)
    pub async fn get_multi_address_balance(&self, addresses: &[String]) -> Result<i64, AppError> {
        let mut total_balance = 0i64;
        
        for address in addresses {
            match self.get_address_balance(address).await {
                Ok(balance) => total_balance += balance,
                Err(e) => {
                    tracing::warn!("Failed to get balance for address {}: {}", address, e);
                    // Continue with other addresses instead of failing completely
                }
            }
        }
        
        Ok(total_balance)
    }

    /// Get user balance by user_id (looks up user's addresses from database)
    pub async fn get_user_balance_by_id(&self, database: &crate::database::Database, user_id: &str) -> Result<i64, AppError> {
        // Get user's Bitcoin addresses from database
        let addresses = database.get_user_addresses(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        if addresses.is_empty() {
            return Ok(0);
        }
        
        // Query blockchain balance for all user addresses
        self.get_multi_address_balance(&addresses).await
    }

    /// Get detailed UTXO information for an address
    pub async fn get_address_utxo_details(&self, address: &str) -> Result<Vec<UtxoInfo>, AppError> {
        let utxos = self.get_address_utxos(address).await?;
        
        let mut utxo_details = Vec::new();
        for utxo in utxos {
            if let (Some(txid), Some(vout), Some(amount)) = (
                utxo.get("txid").and_then(|v| v.as_str()),
                utxo.get("vout").and_then(|v| v.as_u64()),
                utxo.get("amount").and_then(|v| v.as_f64())
            ) {
                utxo_details.push(UtxoInfo {
                    txid: txid.to_string(),
                    vout: vout as u32,
                    amount_sats: (amount * 100_000_000.0) as i64,
                    confirmations: utxo.get("height")
                        .and_then(|v| v.as_u64())
                        .map(|h| h as u32),
                });
            }
        }
        
        Ok(utxo_details)
    }
}

#[derive(Debug, Clone)]
pub struct UtxoInfo {
    pub txid: String,
    pub vout: u32,
    pub amount_sats: i64,
    pub confirmations: Option<u32>,
}