use axum::{extract::State, Json};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    auth::Claims,
    error::APIError,
    telegram_auth::{TelegramAuthData, TelegramAuthService, TelegramUser},
    utils::AppState,
};

#[derive(Deserialize)]
pub struct TelegramLoginRequest {
    pub auth_data: TelegramAuthData,
}

#[derive(Serialize)]
pub struct TelegramLoginResponse {
    pub token: String,
    pub user_id: String,
    pub telegram_id: i64,
}

#[derive(Deserialize)]
pub struct BotServiceRequest {
    pub telegram_user_id: i64,
    pub telegram_username: Option<String>,
    pub command: String,
    pub data: serde_json::Value,
}

#[derive(Serialize)]
pub struct BotServiceResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: Option<String>,
}

/// Telegram user login endpoint
pub async fn telegram_login(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<TelegramLoginRequest>, APIError>,
) -> Result<Json<TelegramLoginResponse>, APIError> {
    let bot_token = std::env::var("BOT_TOKEN")
        .map_err(|_| APIError::Unexpected("Bot token not configured".to_string()))?;

    let auth_service = state.auth_service.lock().await;
    let auth_service = auth_service.as_ref()
        .ok_or(APIError::Unexpected("Auth service not initialized".to_string()))?;

    let telegram_auth = TelegramAuthService::new(bot_token, auth_service.clone());
    let token = telegram_auth.authenticate_telegram_user(payload.auth_data.clone())?;

    // Initialize user in database if needed
    let user_id = format!("tg_{}", payload.auth_data.id);
    if let Some(user_manager) = state.user_manager.lock().await.as_ref() {
        let _ = user_manager.get_user_context(&user_id).await;
    }

    Ok(Json(TelegramLoginResponse {
        token,
        user_id,
        telegram_id: payload.auth_data.id,
    }))
}

/// Bot service proxy endpoint for bitMaskRGB bot
pub async fn bot_service_proxy(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    WithRejection(Json(payload), _): WithRejection<Json<BotServiceRequest>, APIError>,
) -> Result<Json<BotServiceResponse>, APIError> {
    // Verify the request is from the correct Telegram user
    let expected_user_id = format!("tg_{}", payload.telegram_user_id);
    if claims.user_id != expected_user_id {
        return Err(APIError::Forbidden("User ID mismatch".to_string()));
    }

    // Route command to appropriate handler
    let result = match payload.command.as_str() {
        "get_balance" => handle_get_balance(state, &claims, payload.data).await,
        "generate_address" => handle_generate_address(state, &claims, payload.data).await,
        "send_asset" => handle_send_asset(state, &claims, payload.data).await,
        "list_assets" => handle_list_assets(state, &claims, payload.data).await,
        "create_channel" => handle_create_channel(state, &claims, payload.data).await,
        _ => Err(APIError::InvalidRequest(format!("Unknown command: {}", payload.command))),
    };

    match result {
        Ok(data) => Ok(Json(BotServiceResponse {
            success: true,
            data,
            message: None,
        })),
        Err(e) => Ok(Json(BotServiceResponse {
            success: false,
            data: serde_json::Value::Null,
            message: Some(e.to_string()),
        })),
    }
}

async fn handle_get_balance(
    state: Arc<AppState>,
    claims: &Claims,
    data: serde_json::Value,
) -> Result<serde_json::Value, APIError> {
    let asset_id = data.get("asset_id").and_then(|v| v.as_str());
    
    let user_manager = state.user_manager.lock().await;
    let user_manager = user_manager.as_ref()
        .ok_or(APIError::Unexpected("User manager not initialized".to_string()))?;

    let balance = user_manager.get_user_balance(&claims.user_id, asset_id).await?;
    
    Ok(serde_json::json!({
        "balance": balance,
        "asset_id": asset_id
    }))
}

async fn handle_generate_address(
    state: Arc<AppState>,
    claims: &Claims,
    _data: serde_json::Value,
) -> Result<serde_json::Value, APIError> {
    let unlocked_state = state.check_unlocked().await?.clone().unwrap();
    let address = unlocked_state.rgb_get_address()?;

    let user_manager = state.user_manager.lock().await;
    let user_manager = user_manager.as_ref()
        .ok_or(APIError::Unexpected("User manager not initialized".to_string()))?;

    user_manager.save_user_address(&claims.user_id, &address).await?;

    Ok(serde_json::json!({
        "address": address
    }))
}

async fn handle_send_asset(
    _state: Arc<AppState>,
    _claims: &Claims,
    _data: serde_json::Value,
) -> Result<serde_json::Value, APIError> {
    // Implement asset sending logic
    Ok(serde_json::json!({
        "status": "pending",
        "message": "Asset send functionality to be implemented"
    }))
}

async fn handle_list_assets(
    _state: Arc<AppState>,
    _claims: &Claims,
    _data: serde_json::Value,
) -> Result<serde_json::Value, APIError> {
    // Implement asset listing logic
    Ok(serde_json::json!({
        "assets": [],
        "message": "Asset listing functionality to be implemented"
    }))
}

async fn handle_create_channel(
    _state: Arc<AppState>,
    _claims: &Claims,
    _data: serde_json::Value,
) -> Result<serde_json::Value, APIError> {
    // Implement channel creation logic
    Ok(serde_json::json!({
        "status": "pending",
        "message": "Channel creation functionality to be implemented"
    }))
}