use axum::{extract::State, response::Json};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::APIError;
use crate::utils::AppState;

#[derive(Deserialize)]
pub struct AddTestUtxoRequest {
    pub user_id: i64,
    pub amount_sats: u64,
}

#[derive(Serialize)]
pub struct AddTestUtxoResponse {
    pub success: bool,
    pub txid: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct AddTestAddressRequest {
    pub user_id: String,
    pub address: String,
}

#[derive(Serialize)]
pub struct AddTestAddressResponse {
    pub success: bool,
    pub message: String,
}

/// Add test UTXO for a user (for testing purposes only)
pub async fn add_test_utxo(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<AddTestUtxoRequest>, APIError>,
) -> Result<Json<AddTestUtxoResponse>, APIError> {
    let unlocked_state = state.get_unlocked_app_state().await;
    if unlocked_state.is_none() {
        return Err(APIError::LockedNode);
    }
    
    // Initialize database if needed
    if std::env::var("DATABASE_URL").is_ok() {
        if state.database.lock().await.is_none() {
            let _ = crate::utils::initialize_database_after_unlock(&state).await;
        }
    }
    
    let database = state.database.lock().await;
    if let Some(db) = database.as_ref() {
        let txid = format!("test_{}", Uuid::new_v4());
        
        // Add UTXO to user_utxos table
        sqlx::query!(
            "INSERT INTO user_utxos (user_id, txid, vout, amount, address, spent) VALUES ($1, $2, 0, $3, 'test_address', false)",
            payload.user_id,
            txid,
            payload.amount_sats as i64
        )
        .execute(db.pool())
        .await
        .map_err(|e| APIError::Unexpected(format!("Failed to add test UTXO: {}", e)))?;
        
        // Also update ln_user_balances for consistency
        sqlx::query!(
            "INSERT INTO ln_user_balances (user_id, balance, updated_at) VALUES ($1, $2, NOW()) 
             ON CONFLICT (user_id, COALESCE(asset_id, '')) DO UPDATE SET balance = ln_user_balances.balance + $2, updated_at = NOW()",
            payload.user_id.to_string(),
            payload.amount_sats as i64
        )
        .execute(db.pool())
        .await
        .map_err(|e| APIError::Unexpected(format!("Failed to update balance: {}", e)))?;
        
        Ok(Json(AddTestUtxoResponse {
            success: true,
            txid,
            message: format!("Added {} sats to user {}", payload.amount_sats, payload.user_id),
        }))
    } else {
        Err(APIError::Unexpected("Database not available".to_string()))
    }
}

/// Add test Bitcoin address for a user (for testing purposes only)
pub async fn add_test_address(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<AddTestAddressRequest>, APIError>,
) -> Result<Json<AddTestAddressResponse>, APIError> {
    let unlocked_state = state.get_unlocked_app_state().await;
    if unlocked_state.is_none() {
        return Err(APIError::LockedNode);
    }
    
    // Initialize database if needed
    if std::env::var("DATABASE_URL").is_ok() {
        if state.database.lock().await.is_none() {
            let _ = crate::utils::initialize_database_after_unlock(&state).await;
        }
    }
    
    let database = state.database.lock().await;
    if let Some(db) = database.as_ref() {
        // Parse user_id as i64
        let user_id = payload.user_id.parse::<i64>()
            .map_err(|_| APIError::Unexpected("Invalid user_id format".to_string()))?;
        
        // Add address to user_addresses table
        sqlx::query!(
            "INSERT INTO user_addresses (user_id, address) VALUES ($1, $2) ON CONFLICT (user_id, address) DO NOTHING",
            user_id,
            payload.address
        )
        .execute(db.pool())
        .await
        .map_err(|e| APIError::Unexpected(format!("Failed to add address: {}", e)))?;
        
        Ok(Json(AddTestAddressResponse {
            success: true,
            message: format!("Added address {} for user {}", payload.address, payload.user_id),
        }))
    } else {
        Err(APIError::Unexpected("Database not available".to_string()))
    }
}