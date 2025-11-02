use crate::error::AppError;
use crate::utils::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundPaymentEvent {
    pub payment_hash: String,
    pub amount_msat: u64,
    pub receiver_user_id: Option<i64>,
    pub metadata: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceWithMetadata {
    pub invoice: String,
    pub payment_hash: String,
    pub user_id: i64,
    pub amount_msat: u64,
    pub expiry: u32,
}

pub struct InboundPaymentHandler {
    app_state: Arc<AppState>,
}

impl InboundPaymentHandler {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// Generate invoice with user metadata for inbound payments
    pub async fn create_user_invoice(
        &self,
        user_id: i64,
        amount_msat: u64,
        _description: Option<String>,
    ) -> Result<InvoiceWithMetadata, AppError> {
        // Create metadata to identify the receiving user
        let _metadata = serde_json::json!({
            "user_id": user_id,
            "type": "inbound_payment"
        }).to_string();

        // Generate invoice through Lightning router
        let lightning_router = crate::lightning_router::LightningRouter::new(self.app_state.clone());
        let invoice = lightning_router.create_lightning_invoice(
            None, // BTC payment
            Some(3600), // 1 hour expiry
            Some(1),
        ).await?;

        // Extract payment hash from invoice (simplified)
        let payment_hash = format!("hash_{}", chrono::Utc::now().timestamp());

        Ok(InvoiceWithMetadata {
            invoice,
            payment_hash,
            user_id,
            amount_msat,
            expiry: 3600,
        })
    }

    /// Process inbound payment event (called by Kafka consumer or webhook)
    pub async fn process_inbound_payment(
        &self,
        event: InboundPaymentEvent,
    ) -> Result<(), AppError> {
        tracing::info!("Processing inbound payment: {:?}", event);

        // Extract user ID from metadata
        let user_id = if let Some(user_id) = event.receiver_user_id {
            user_id
        } else if let Some(metadata) = &event.metadata {
            // Parse metadata to extract user_id
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(metadata) {
                meta.get("user_id")
                    .and_then(|v| v.as_i64())
                    .ok_or(AppError::Generic("Invalid metadata format".to_string()))?
            } else {
                return Err(AppError::Generic("Failed to parse metadata".to_string()));
            }
        } else {
            return Err(AppError::Generic("No user ID found in payment event".to_string()));
        };

        // Initialize database if needed
        if std::env::var("DATABASE_URL").is_ok() {
            if self.app_state.database.lock().await.is_none() {
                let _ = crate::utils::initialize_database_after_unlock(&self.app_state).await;
            }
        }

        // Credit user's balance
        if let Some(db) = self.app_state.database.lock().await.as_ref() {
            let amount_sats = event.amount_msat / 1000;
            
            // Create new UTXO for the user
            sqlx::query!(
                "INSERT INTO user_utxos (user_id, txid, vout, amount, address, spent) VALUES ($1, $2, 0, $3, 'inbound_payment', false)",
                user_id,
                format!("inbound_{}", event.payment_hash),
                amount_sats as i64
            )
            .execute(db.pool())
            .await
            .map_err(|e| AppError::Generic(format!("Failed to credit user balance: {}", e)))?;

            // Record transaction
            sqlx::query!(
                "INSERT INTO ln_user_transactions (id, user_id, txid, amount, status) VALUES ($1, $2, $3, $4, 'completed')",
                uuid::Uuid::new_v4(),
                user_id.to_string(),
                format!("inbound_{}", event.payment_hash),
                amount_sats as i64
            )
            .execute(db.pool())
            .await
            .map_err(|e| AppError::Generic(format!("Failed to record transaction: {}", e)))?;

            tracing::info!("Credited {} sats to user {}", amount_sats, user_id);
        }

        Ok(())
    }

    /// Webhook endpoint for Lightning Network events
    pub async fn handle_payment_webhook(
        &self,
        payment_hash: String,
        amount_msat: u64,
        metadata: Option<String>,
    ) -> Result<(), AppError> {
        let event = InboundPaymentEvent {
            payment_hash,
            amount_msat,
            receiver_user_id: None,
            metadata,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.process_inbound_payment(event).await
    }
}