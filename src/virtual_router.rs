use bitcoin::secp256k1::PublicKey;
use bitcoin::hashes::Hash;
use lightning::ln::{PaymentHash, PaymentPreimage};
use lightning::routing::router::Route;
use lightning::sign::EntropySource;
use rgb_lib::ContractId;
use std::sync::Arc;
use crate::virtual_htlc::{VirtualHtlcManager, RgbTransfer, VirtualSettlement};
use crate::virtual_balance::VirtualBalanceManager;
use crate::utils::UnlockedAppState;

/// Virtual router for HTLC routing between virtual nodes
pub struct VirtualRouter {
    htlc_manager: Arc<VirtualHtlcManager>,
    balance_manager: Arc<VirtualBalanceManager>,
    unlocked_state: Arc<UnlockedAppState>,
}

impl VirtualRouter {
    pub fn new(
        htlc_manager: Arc<VirtualHtlcManager>,
        balance_manager: Arc<VirtualBalanceManager>,
        unlocked_state: Arc<UnlockedAppState>,
    ) -> Self {
        Self {
            htlc_manager,
            balance_manager,
            unlocked_state,
        }
    }

    /// Send payment between virtual nodes (BTC + RGB)
    pub async fn send_virtual_payment(
        &self,
        from_virtual_node: PublicKey,
        to_virtual_node: PublicKey,
        btc_amount_msat: u64,
        rgb_transfer: Option<RgbTransfer>,
        payment_hash: PaymentHash,
    ) -> Result<VirtualPaymentResult, VirtualRouterError> {
        // 1. Check sufficient balance
        self.balance_manager
            .check_sufficient_balance(&from_virtual_node, btc_amount_msat, rgb_transfer.as_ref())
            .await
            .map_err(|e| VirtualRouterError::InsufficientBalance(e.to_string()))?;

        // 2. Create virtual HTLC
        self.htlc_manager
            .create_virtual_htlc(
                payment_hash,
                from_virtual_node,
                to_virtual_node,
                btc_amount_msat,
                rgb_transfer.clone(),
            )
            .await
            .map_err(|e| VirtualRouterError::HtlcCreation(e.to_string()))?;

        // 3. For now, auto-settle (in real implementation, this would wait for preimage)
        // TODO: Integrate with actual Lightning routing
        let preimage = PaymentPreimage([42u8; 32]); // Mock preimage
        
        let settlement = self.settle_virtual_payment(payment_hash, preimage).await?;

        Ok(VirtualPaymentResult {
            payment_hash,
            preimage,
            settlement,
        })
    }

    /// Settle virtual payment with preimage
    pub async fn settle_virtual_payment(
        &self,
        payment_hash: PaymentHash,
        preimage: PaymentPreimage,
    ) -> Result<VirtualSettlement, VirtualRouterError> {
        // 1. Settle virtual HTLC
        let settlement = self.htlc_manager
            .settle_virtual_htlc(payment_hash, preimage)
            .await
            .map_err(|e| VirtualRouterError::Settlement(e.to_string()))?;

        // 2. Apply balance changes atomically
        self.balance_manager
            .apply_settlement(&settlement)
            .await
            .map_err(|e| VirtualRouterError::BalanceUpdate(e.to_string()))?;

        tracing::info!(
            "Virtual payment settled: {} -> {} ({} msat BTC + {:?} RGB)",
            settlement.from_virtual_node,
            settlement.to_virtual_node,
            settlement.btc_settled,
            settlement.rgb_settled
        );

        Ok(settlement)
    }

    /// Create Lightning invoice for virtual node
    pub async fn create_virtual_invoice(
        &self,
        virtual_node: PublicKey,
        btc_amount_msat: Option<u64>,
        rgb_request: Option<RgbInvoiceRequest>,
        expiry_sec: u32,
    ) -> Result<VirtualInvoice, VirtualRouterError> {
        // Generate payment hash for virtual invoice
        let preimage = PaymentPreimage(self.unlocked_state.keys_manager.get_secure_random_bytes());
        let payment_hash = PaymentHash(
            bitcoin::hashes::sha256::Hash::hash(&preimage.0).to_byte_array()
        );

        let virtual_invoice = VirtualInvoice {
            virtual_node,
            payment_hash,
            preimage,
            btc_amount_msat,
            rgb_request: rgb_request.clone(),
            expiry_sec,
        };

        tracing::info!(
            "Created virtual invoice for {}: {} msat BTC + {:?} RGB",
            virtual_node,
            btc_amount_msat.unwrap_or(0),
            rgb_request
        );

        Ok(virtual_invoice)
    }

    /// Pay virtual invoice
    pub async fn pay_virtual_invoice(
        &self,
        from_virtual_node: PublicKey,
        invoice: &VirtualInvoice,
    ) -> Result<VirtualPaymentResult, VirtualRouterError> {
        let rgb_transfer = invoice.rgb_request.as_ref().map(|req| RgbTransfer {
            contract_id: req.contract_id,
            amount: req.amount,
        });

        self.send_virtual_payment(
            from_virtual_node,
            invoice.virtual_node,
            invoice.btc_amount_msat.unwrap_or(0),
            rgb_transfer,
            invoice.payment_hash,
        ).await
    }
}

#[derive(Clone, Debug)]
pub struct VirtualPaymentResult {
    pub payment_hash: PaymentHash,
    pub preimage: PaymentPreimage,
    pub settlement: VirtualSettlement,
}

#[derive(Clone, Debug)]
pub struct VirtualInvoice {
    pub virtual_node: PublicKey,
    pub payment_hash: PaymentHash,
    pub preimage: PaymentPreimage,
    pub btc_amount_msat: Option<u64>,
    pub rgb_request: Option<RgbInvoiceRequest>,
    pub expiry_sec: u32,
}

#[derive(Clone, Debug)]
pub struct RgbInvoiceRequest {
    pub contract_id: ContractId,
    pub amount: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum VirtualRouterError {
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),
    #[error("HTLC creation failed: {0}")]
    HtlcCreation(String),
    #[error("Settlement failed: {0}")]
    Settlement(String),
    #[error("Balance update failed: {0}")]
    BalanceUpdate(String),
    #[error("Routing failed: {0}")]
    RoutingFailed(String),
}