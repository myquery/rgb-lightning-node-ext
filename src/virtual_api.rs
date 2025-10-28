use axum::{extract::State, response::Json};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::APIError;
use crate::utils::AppState;

#[derive(Deserialize)]
pub struct VirtualRgbInvoiceRequest {
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
    WithRejection(Json(req), rejection): WithRejection<Json<VirtualSendPaymentRequest>, APIError>,
) -> Result<Json<VirtualSendPaymentResponse>, APIError> {
    let response = crate::routes::send_payment(State(app_state), WithRejection(Json(crate::routes::SendPaymentRequest {
        invoice: req.invoice,
        amt_msat: req.amt_msat,
    }), rejection)).await?;
    
    Ok(Json(VirtualSendPaymentResponse {
        payment_id: response.payment_id.clone(),
        payment_hash: response.payment_hash.clone(),
        status: format!("{:?}", response.status),
    }))
}

pub async fn virtual_assetbalance(
    State(app_state): State<Arc<AppState>>,
    WithRejection(Json(payload), rejection): WithRejection<Json<serde_json::Value>, APIError>,
) -> Result<Json<VirtualAssetBalanceResponse>, APIError> {
    let response = crate::routes::asset_balance(State(app_state), WithRejection(Json(payload), rejection)).await?;
    
    Ok(Json(VirtualAssetBalanceResponse {
        settled: response.settled,
        future: response.future,
        spendable: response.spendable,
        offchain_outbound: response.offchain_outbound,
        offchain_inbound: response.offchain_inbound,
    }))
}