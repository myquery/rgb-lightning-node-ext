use axum::{extract::State, response::Json};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::APIError;
use crate::utils::AppState;

#[derive(Deserialize)]
pub struct VirtualRgbInvoiceRequest {
    pub user_id: String,  // bitMaskRGB sends user_id
    pub asset_id: Option<String>,
    pub duration_seconds: Option<u32>,
    pub min_confirmations: Option<u8>,
}

#[derive(Serialize)]
pub struct VirtualRgbInvoiceResponse {
    pub recipient_id: String,
    pub invoice: String,
    pub expiration_timestamp: Option<i64>,
    pub batch_transfer_idx: i32,
}

#[derive(Deserialize)]
pub struct VirtualSendPaymentRequest {
    pub user_id: String,  // bitMaskRGB sends user_id
    pub invoice: String,
    pub amt_msat: Option<u64>,
}

#[derive(Serialize)]
pub struct VirtualSendPaymentResponse {
    pub payment_id: String,
    pub payment_hash: Option<String>,
    pub status: String,
}

#[derive(Deserialize)]
pub struct VirtualAssetBalanceRequest {
    pub user_id: String,  // bitMaskRGB sends user_id
    pub asset_id: String,
}

#[derive(Serialize)]
pub struct VirtualAssetBalanceResponse {
    pub settled: u64,
    pub future: u64,
    pub spendable: u64,
    pub offchain_outbound: u64,
    pub offchain_inbound: u64,
}

pub async fn virtual_rgbinvoice(
    State(app_state): State<Arc<AppState>>,
    WithRejection(Json(req), rejection): WithRejection<Json<VirtualRgbInvoiceRequest>, APIError>,
) -> Result<Json<VirtualRgbInvoiceResponse>, APIError> {
    tracing::info!("Virtual RGB invoice request for user_id: {}", req.user_id);
    
    let response = crate::routes::rgb_invoice(State(app_state), WithRejection(Json(crate::routes::RgbInvoiceRequest {
        asset_id: req.asset_id,
        duration_seconds: req.duration_seconds,
        min_confirmations: req.min_confirmations.unwrap_or(1),
    }), rejection)).await?;
    
    Ok(Json(VirtualRgbInvoiceResponse {
        recipient_id: response.recipient_id.clone(),
        invoice: response.invoice.clone(),
        expiration_timestamp: response.expiration_timestamp,
        batch_transfer_idx: response.batch_transfer_idx,
    }))
}

pub async fn virtual_sendpayment(
    State(app_state): State<Arc<AppState>>,
    WithRejection(Json(req), _rejection): WithRejection<Json<VirtualSendPaymentRequest>, APIError>,
) -> Result<Json<VirtualSendPaymentResponse>, APIError> {
    tracing::info!("Virtual send payment request for user_id: {}", req.user_id);
    
    // Use Lightning router for external payments through master node
    let lightning_router = crate::lightning_router::LightningRouter::new(app_state);
    
    match lightning_router.send_lightning_payment(req.invoice, req.amt_msat).await {
        Ok(response) => Ok(Json(VirtualSendPaymentResponse {
            payment_id: response.payment_id,
            payment_hash: response.payment_hash,
            status: response.status,
        })),
        Err(e) => Err(APIError::FailedPayment(e.to_string())),
    }
}

pub async fn virtual_assetbalance(
    State(app_state): State<Arc<AppState>>,
    WithRejection(Json(req), rejection): WithRejection<Json<VirtualAssetBalanceRequest>, APIError>,
) -> Result<Json<VirtualAssetBalanceResponse>, APIError> {
    tracing::info!("Virtual asset balance request for user_id: {} asset_id: {}", req.user_id, req.asset_id);
    
    // Convert to the format expected by asset_balance endpoint
    let payload = serde_json::json!({
        "asset_id": req.asset_id
    });
    
    let response = crate::routes::asset_balance(State(app_state), WithRejection(Json(payload), rejection)).await?;
    
    Ok(Json(VirtualAssetBalanceResponse {
        settled: response.settled,
        future: response.future,
        spendable: response.spendable,
        offchain_outbound: response.offchain_outbound,
        offchain_inbound: response.offchain_inbound,
    }))
}