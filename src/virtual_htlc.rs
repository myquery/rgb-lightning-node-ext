use bitcoin::secp256k1::PublicKey;
use bitcoin::hashes::Hash;
use lightning::ln::{PaymentHash, PaymentPreimage};
use rgb_lib::ContractId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Virtual HTLC that settles both BTC and RGB atomically
#[derive(Clone, Debug)]
pub struct VirtualHtlc {
    pub payment_hash: PaymentHash,
    pub from_virtual_node: PublicKey,
    pub to_virtual_node: PublicKey,
    pub btc_amount_msat: u64,
    pub rgb_transfer: Option<RgbTransfer>,
    pub status: VirtualHtlcStatus,
}

#[derive(Clone, Debug)]
pub struct RgbTransfer {
    pub contract_id: ContractId,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VirtualHtlcStatus {
    Pending,
    Settled,
    Failed,
}

/// Virtual HTLC settlement manager
pub struct VirtualHtlcManager {
    pending_htlcs: Arc<Mutex<HashMap<PaymentHash, VirtualHtlc>>>,
    virtual_channel_manager: Arc<crate::virtual_channel::VirtualChannelManager>,
}

impl VirtualHtlcManager {
    pub fn new(virtual_channel_manager: Arc<crate::virtual_channel::VirtualChannelManager>) -> Self {
        Self {
            pending_htlcs: Arc::new(Mutex::new(HashMap::new())),
            virtual_channel_manager,
        }
    }

    /// Create virtual HTLC for BTC + RGB transfer
    pub async fn create_virtual_htlc(
        &self,
        payment_hash: PaymentHash,
        from_virtual_node: PublicKey,
        to_virtual_node: PublicKey,
        btc_amount_msat: u64,
        rgb_transfer: Option<RgbTransfer>,
    ) -> Result<(), VirtualHtlcError> {
        let virtual_htlc = VirtualHtlc {
            payment_hash,
            from_virtual_node,
            to_virtual_node,
            btc_amount_msat,
            rgb_transfer,
            status: VirtualHtlcStatus::Pending,
        };

        let mut pending = self.pending_htlcs.lock().await;
        pending.insert(payment_hash, virtual_htlc);

        // Map payment to virtual nodes in database
        let payment_hash_hex = payment_hash.0.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        self.virtual_channel_manager
            .map_payment_to_virtual_node(
                &payment_hash_hex,
                &from_virtual_node.to_string(),
1, // TODO: Get actual user_id
                false,
            )
            .await
            .map_err(|e| VirtualHtlcError::Database(e.to_string()))?;

        self.virtual_channel_manager
            .map_payment_to_virtual_node(
                &payment_hash_hex,
                &to_virtual_node.to_string(),
2, // TODO: Get actual user_id
                true,
            )
            .await
            .map_err(|e| VirtualHtlcError::Database(e.to_string()))?;

        Ok(())
    }

    /// Settle virtual HTLC with preimage - settles BOTH BTC and RGB atomically
    pub async fn settle_virtual_htlc(
        &self,
        payment_hash: PaymentHash,
        preimage: PaymentPreimage,
    ) -> Result<VirtualSettlement, VirtualHtlcError> {
        let mut pending = self.pending_htlcs.lock().await;
        
        let mut virtual_htlc = pending
            .get_mut(&payment_hash)
            .ok_or(VirtualHtlcError::HtlcNotFound)?
            .clone();

        // Verify preimage matches hash
        if PaymentHash(bitcoin::hashes::sha256::Hash::hash(&preimage.0).to_byte_array()) != payment_hash {
            return Err(VirtualHtlcError::InvalidPreimage);
        }

        // Atomic settlement: BTC + RGB
        let settlement = VirtualSettlement {
            payment_hash,
            preimage,
            btc_settled: virtual_htlc.btc_amount_msat,
            rgb_settled: virtual_htlc.rgb_transfer.clone(),
            from_virtual_node: virtual_htlc.from_virtual_node,
            to_virtual_node: virtual_htlc.to_virtual_node,
        };

        // Update HTLC status
        virtual_htlc.status = VirtualHtlcStatus::Settled;
        pending.insert(payment_hash, virtual_htlc);

        tracing::info!(
            "Virtual HTLC settled: {} msat BTC + {:?} RGB between {} -> {}",
            settlement.btc_settled,
            settlement.rgb_settled,
            settlement.from_virtual_node,
            settlement.to_virtual_node
        );

        Ok(settlement)
    }

    /// Fail virtual HTLC
    pub async fn fail_virtual_htlc(&self, payment_hash: PaymentHash) -> Result<(), VirtualHtlcError> {
        let mut pending = self.pending_htlcs.lock().await;
        
        if let Some(virtual_htlc) = pending.get_mut(&payment_hash) {
            virtual_htlc.status = VirtualHtlcStatus::Failed;
            tracing::info!("Virtual HTLC failed: {}", payment_hash);
        }

        Ok(())
    }

    /// Get pending virtual HTLCs for a virtual node
    pub async fn get_pending_htlcs(&self, virtual_node: &PublicKey) -> Vec<VirtualHtlc> {
        let pending = self.pending_htlcs.lock().await;
        pending
            .values()
            .filter(|htlc| {
                htlc.status == VirtualHtlcStatus::Pending &&
                (htlc.from_virtual_node == *virtual_node || htlc.to_virtual_node == *virtual_node)
            })
            .cloned()
            .collect()
    }
}

/// Result of virtual HTLC settlement
#[derive(Clone, Debug)]
pub struct VirtualSettlement {
    pub payment_hash: PaymentHash,
    pub preimage: PaymentPreimage,
    pub btc_settled: u64,
    pub rgb_settled: Option<RgbTransfer>,
    pub from_virtual_node: PublicKey,
    pub to_virtual_node: PublicKey,
}

#[derive(Debug, thiserror::Error)]
pub enum VirtualHtlcError {
    #[error("HTLC not found")]
    HtlcNotFound,
    #[error("Invalid preimage")]
    InvalidPreimage,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Insufficient balance")]
    InsufficientBalance,
}

impl VirtualSettlement {
    /// Check if this settlement includes RGB transfer
    pub fn has_rgb_transfer(&self) -> bool {
        self.rgb_settled.is_some()
    }

    /// Get RGB contract ID if present
    pub fn rgb_contract_id(&self) -> Option<ContractId> {
        self.rgb_settled.as_ref().map(|t| t.contract_id)
    }

    /// Get RGB amount if present
    pub fn rgb_amount(&self) -> Option<u64> {
        self.rgb_settled.as_ref().map(|t| t.amount)
    }
}