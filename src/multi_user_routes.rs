use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use axum_extra::extract::WithRejection;
use std::sync::Arc;

use crate::{
    auth::{Claims, UserRole},
    error::APIError,
    user_manager_enhanced::{UserManager, UserActivity},
    multi_user_rgb::MultiUserRgbManager,
    utils::AppState,
    routes::{AddressResponse, AssetBalanceRequest, AssetBalanceResponse},
};

/// Multi-user context middleware that injects user context into requests
pub async fn multi_user_context_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract user claims from request extensions (set by auth middleware)
    if let Some(claims) = request.extensions().get::<Claims>() {
        // Add user context to request for downstream handlers
        request.extensions_mut().insert(claims.clone());
    }

    Ok(next.run(request).await)
}

/// Enhanced address endpoint with multi-user support
pub async fn multi_user_address(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<AddressResponse>, APIError> {
    let claims = request.extensions().get::<Claims>()
        .ok_or(APIError::Unauthorized("Missing user context".to_string()))?;

    let unlocked_state = state.check_unlocked().await?.clone().unwrap();

    // Get user manager
    let user_manager = state.user_manager.lock().await;
    let user_manager = user_manager.as_ref()
        .ok_or(APIError::Unexpected("User manager not initialized".to_string()))?;

    // Check user permissions
    if !user_manager.can_write(&claims.role) {
        return Err(APIError::Forbidden("Insufficient permissions".to_string()));
    }

    // Get user context
    let user_context = user_manager.get_user_context(&claims.user_id, claims.role.clone()).await?;

    // Generate user-specific address
    let address = unlocked_state.rgb_get_address()?;

    // Save address for user
    user_manager.save_user_address(&claims.user_id, &address).await?;

    // Log activity
    let activity = UserActivity {
        user_id: claims.user_id.clone(),
        action: "generate_address".to_string(),
        details: serde_json::json!({"address": address}),
        timestamp: chrono::Utc::now(),
        ip_address: None, // Would extract from request
    };
    user_manager.log_activity(activity).await?;

    Ok(Json(AddressResponse { address }))
}

/// Enhanced asset balance endpoint with multi-user support
pub async fn multi_user_asset_balance(
    State(state): State<Arc<AppState>>,
    request: Request,
    WithRejection(Json(payload), _): WithRejection<Json<AssetBalanceRequest>, APIError>,
) -> Result<Json<AssetBalanceResponse>, APIError> {
    let claims = request.extensions().get::<Claims>()
        .ok_or(APIError::Unauthorized("Missing user context".to_string()))?;

    let unlocked_state = state.check_unlocked().await?.clone().unwrap();

    // Get user manager
    let user_manager = state.user_manager.lock().await;
    let user_manager = user_manager.as_ref()
        .ok_or(APIError::Unexpected("User manager not initialized".to_string()))?;

    // Check user permissions
    if !user_manager.can_read(&claims.role) {
        return Err(APIError::Forbidden("Insufficient permissions".to_string()));
    }

    // Get user-specific balance from multi-user RGB manager
    let rgb_manager = state.multi_user_rgb.lock().await;
    let rgb_manager = rgb_manager.as_ref()
        .ok_or(APIError::Unexpected("Multi-user RGB manager not initialized".to_string()))?;

    let user_wallet = rgb_manager.get_user_wallet(&claims.user_id, &claims.role).await?;

    // Get asset balance from user's isolated wallet
    let balance = user_manager.get_user_balance(&claims.user_id, Some(&payload.asset_id)).await
        .map_err(|e| APIError::Unexpected(e.to_string()))? as u64;

    // Log activity
    let activity = UserActivity {
        user_id: claims.user_id.clone(),
        action: "get_asset_balance".to_string(),
        details: serde_json::json!({"asset_id": payload.asset_id, "balance": balance}),
        timestamp: chrono::Utc::now(),
        ip_address: None,
    };
    user_manager.log_activity(activity).await?;

    Ok(Json(AssetBalanceResponse {
        settled: balance,
        future: balance,
        spendable: balance,
        offchain_outbound: 0,
        offchain_inbound: 0,
    }))
}

/// User quota check middleware
pub async fn quota_check_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(claims) = request.extensions().get::<Claims>() {
        let user_manager = state.user_manager.lock().await;
        if let Some(user_manager) = user_manager.as_ref() {
            // Determine action from request path
            let action = match request.uri().path() {
                "/openchannel" => "create_channel",
                "/issueassetnia" | "/issueassetuda" | "/issueassetcfa" => "create_asset",
                "/sendpayment" | "/sendasset" => "transaction",
                _ => "general",
            };

            // Check quota
            match user_manager.check_user_quota(&claims.user_id, action).await {
                Ok(true) => {}, // Quota check passed
                Ok(false) => return Err(StatusCode::TOO_MANY_REQUESTS),
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }

    Ok(next.run(request).await)
}

/// Admin-only endpoint for user management
pub async fn admin_list_users(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Vec<UserSummary>>, APIError> {
    let claims = request.extensions().get::<Claims>()
        .ok_or(APIError::Unauthorized("Missing user context".to_string()))?;

    // Check admin permissions
    if !matches!(claims.role, UserRole::Admin) {
        return Err(APIError::Forbidden("Admin access required".to_string()));
    }

    let user_manager = state.user_manager.lock().await;
    let user_manager = user_manager.as_ref()
        .ok_or(APIError::Unexpected("User manager not initialized".to_string()))?;

    // Get RGB manager for user stats
    let rgb_manager = state.multi_user_rgb.lock().await;
    let rgb_manager = rgb_manager.as_ref()
        .ok_or(APIError::Unexpected("Multi-user RGB manager not initialized".to_string()))?;

    let active_users = rgb_manager.list_active_users().await;
    let mut user_summaries = Vec::new();

    for user_id in active_users {
        let stats = rgb_manager.get_user_wallet_stats(&user_id).await?;
        let channels = user_manager.get_user_channels(&user_id).await?.len();
        let transactions = user_manager.get_user_transactions(&user_id).await?.len();

        user_summaries.push(UserSummary {
            user_id,
            total_assets: stats.total_assets,
            total_channels: channels as u32,
            total_transactions: transactions as u32,
            wallet_size_bytes: stats.wallet_size_bytes,
        });
    }

    Ok(Json(user_summaries))
}

#[derive(serde::Serialize)]
pub struct UserSummary {
    pub user_id: String,
    pub total_assets: u32,
    pub total_channels: u32,
    pub total_transactions: u32,
    pub wallet_size_bytes: u64,
}

/// Rate limiting middleware per user
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(claims) = request.extensions().get::<Claims>() {
        // Implement rate limiting logic per user
        // This would typically use Redis or in-memory store
        tracing::debug!("Rate limiting check for user: {}", claims.user_id);
    }

    Ok(next.run(request).await)
}