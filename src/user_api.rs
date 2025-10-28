use serde::{Deserialize, Serialize};
use axum::{extract::State, response::Json, routing::post, Router};
use std::sync::Arc;
use crate::telegram_integration::TelegramIntegration;
use crate::utils::AppState;

#[derive(Debug, Deserialize)]
pub struct UserAssetBalanceRequest {
    pub user_id: String,
    pub asset_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UserRgbInvoiceRequest {
    pub user_id: String,
    pub asset_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UserSendAssetRequest {
    pub user_id: String,
    pub invoice: String,
    pub asset_id: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub fn user_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/assetbalance", post(handle_asset_balance))
        .route("/rgbinvoice", post(handle_rgb_invoice))
        .route("/sendasset", post(handle_send_asset))
}

async fn handle_asset_balance(
    State(_app_state): State<Arc<AppState>>,
    Json(req): Json<UserAssetBalanceRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    let user_id: i64 = req.user_id.parse().unwrap_or(0);
    
    let balance = get_user_asset_balance(user_id, &req.asset_id).await;
    
    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({ "balance": balance })),
        error: None,
    })
}

async fn handle_rgb_invoice(
    State(_app_state): State<Arc<AppState>>,
    Json(req): Json<UserRgbInvoiceRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    let user_id: i64 = req.user_id.parse().unwrap_or(0);
    
    let invoice = generate_user_rgb_invoice(user_id, &req.asset_id).await;
    
    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({ "invoice": invoice })),
        error: None,
    })
}

async fn handle_send_asset(
    State(_app_state): State<Arc<AppState>>,
    Json(req): Json<UserSendAssetRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    let user_id: i64 = req.user_id.parse().unwrap_or(0);
    
    let txid = send_user_asset(user_id, &req.invoice, &req.asset_id, req.amount).await;
    
    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({ "txid": txid })),
        error: None,
    })
}

// Mock implementations - replace with real RGB operations
async fn get_user_asset_balance(user_id: i64, asset_id: &str) -> u64 {
    // Mock balance based on user_id and asset_id
    match asset_id {
        "USDT" => ((user_id % 1000) * 1000) as u64,
        "BTC" => ((user_id % 100) * 100000) as u64,
        _ => 0,
    }
}

async fn generate_user_rgb_invoice(user_id: i64, asset_id: &str) -> String {
    format!("rgb:{}:any:user_{}:blinding_{}", asset_id, user_id, chrono::Utc::now().timestamp())
}

async fn send_user_asset(user_id: i64, _invoice: &str, asset_id: &str, amount: u64) -> String {
    format!("txid_{}_{}_{}_{}", user_id, asset_id, amount, chrono::Utc::now().timestamp())
}