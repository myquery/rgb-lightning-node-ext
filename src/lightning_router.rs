use crate::error::AppError;
use crate::utils::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningPaymentRequest {
    pub invoice: String,
    pub amt_msat: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningPaymentResponse {
    pub payment_id: String,
    pub payment_hash: Option<String>,
    pub status: String,
}

pub struct LightningRouter {
    app_state: Arc<AppState>,
}

impl LightningRouter {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// Route external payment through master Lightning node
    pub async fn send_lightning_payment(
        &self,
        invoice: String,
        amount_msat: Option<u64>,
    ) -> Result<LightningPaymentResponse, AppError> {
        use axum::extract::State;
        use axum_extra::extract::WithRejection;
        use axum::response::Json;
        
        let request = crate::routes::SendPaymentRequest {
            invoice,
            amt_msat: amount_msat,
        };

        // Call the actual Lightning Network payment via the existing route
        match crate::routes::send_payment(
            State(self.app_state.clone()),
            WithRejection(Json(request), std::marker::PhantomData)
        ).await {
            Ok(Json(response)) => Ok(LightningPaymentResponse {
                payment_id: response.payment_id,
                payment_hash: response.payment_hash,
                status: format!("{:?}", response.status),
            }),
            Err(e) => Err(AppError::Generic(format!("Lightning payment failed: {}", e))),
        }
    }

    /// Generate Lightning invoice through master node
    pub async fn create_lightning_invoice(
        &self,
        asset_id: Option<String>,
        duration_seconds: Option<u32>,
        min_confirmations: Option<u8>,
    ) -> Result<String, AppError> {
        use axum::extract::State;
        use axum_extra::extract::WithRejection;
        use axum::response::Json;
        
        let request = crate::routes::RgbInvoiceRequest {
            asset_id,
            duration_seconds,
            min_confirmations: min_confirmations.unwrap_or(1),
        };

        match crate::routes::rgb_invoice(
            State(self.app_state.clone()),
            WithRejection(Json(request), std::marker::PhantomData)
        ).await {
            Ok(Json(response)) => Ok(response.invoice),
            Err(e) => Err(AppError::Generic(format!("Invoice creation failed: {}", e))),
        }
    }

    /// Get new address from master node
    pub async fn get_new_address(&self) -> Result<String, AppError> {
        use axum::extract::State;
        use axum_extra::extract::WithRejection;
        use axum::response::Json;
        
        match crate::routes::address(
            State(self.app_state.clone()),
            WithRejection(Json(serde_json::json!({})), std::marker::PhantomData)
        ).await {
            Ok(Json(response)) => Ok(response.address),
            Err(e) => Err(AppError::Generic(format!("Address generation failed: {}", e))),
        }
    }
}