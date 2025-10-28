use crate::database::Database;
use crate::error::AppError;
use crate::auth::{UserRole, Claims};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct UserManager {
    database: Database,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub role: UserRole,
    pub balances: HashMap<String, i64>,
    pub addresses: Vec<String>,
    pub quotas: UserQuotas,
    pub settings: UserSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserQuotas {
    pub max_channels: u32,
    pub max_assets: u32,
    pub max_transactions_per_day: u32,
    pub max_balance_btc: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettings {
    pub default_fee_rate: u64,
    pub auto_channel_accept: bool,
    pub notification_preferences: NotificationPreferences,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: String,
    pub action: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
}

impl Default for UserQuotas {
    fn default() -> Self {
        Self {
            max_channels: 10,
            max_assets: 100,
            max_transactions_per_day: 1000,
            max_balance_btc: 1_000_000, // 0.01 BTC in sats
        }
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            default_fee_rate: 1,
            auto_channel_accept: false,
            notification_preferences: NotificationPreferences {
                email_notifications: false,
                webhook_url: None,
            },
        }
    }
}

impl UserManager {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Extract user context from JWT claims (preferred method)
    pub fn extract_user_context_from_claims(claims: &Claims) -> UserContext {
        UserContext {
            user_id: claims.user_id.clone(),
            role: claims.role.clone(),
            balances: HashMap::new(),
            addresses: Vec::new(),
            quotas: UserQuotas::default(),
            settings: UserSettings::default(),
        }
    }

    /// Extract user_id from request body or headers (fallback method)
    pub fn extract_user_id(body: &serde_json::Value, headers: &axum::http::HeaderMap) -> Option<String> {
        // Try to get user_id from request body
        if let Some(user_id) = body.get("user_id").and_then(|v| v.as_str()) {
            return Some(user_id.to_string());
        }

        // Try to get user_id from headers
        if let Some(user_id) = headers.get("x-user-id").and_then(|v| v.to_str().ok()) {
            return Some(user_id.to_string());
        }

        None
    }

    /// Get or create user context with full profile
    pub async fn get_user_context(&self, user_id: &str, role: UserRole) -> Result<UserContext, AppError> {
        // Ensure user wallet exists
        if self.database.get_user_wallet(user_id).await?.is_none() {
            self.create_user_wallet(user_id, &role).await?;
        }

        let balances = self.database.get_user_balances(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        
        let addresses = self.database.get_user_addresses(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let quotas = self.get_user_quotas(user_id).await
            .unwrap_or_else(|_| UserQuotas::default());

        let settings = self.get_user_settings(user_id).await
            .unwrap_or_else(|_| UserSettings::default());

        Ok(UserContext {
            user_id: user_id.to_string(),
            role,
            balances,
            addresses,
            quotas,
            settings,
        })
    }

    /// Create new user wallet with role-based configuration
    async fn create_user_wallet(&self, user_id: &str, role: &UserRole) -> Result<(), AppError> {
        // Generate encrypted mnemonic for user (simplified)
        let mnemonic_encrypted = format!("encrypted_mnemonic_for_{}", user_id);
        let derivation_path = format!("m/84'/1'/0'"); // Testnet derivation path

        self.database.create_user_wallet(user_id, &mnemonic_encrypted, &derivation_path).await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Initialize with zero balances
        self.database.update_user_balance(user_id, None, 0).await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Set role-based quotas
        let quotas = match role {
            UserRole::Admin => UserQuotas {
                max_channels: u32::MAX,
                max_assets: u32::MAX,
                max_transactions_per_day: u32::MAX,
                max_balance_btc: u64::MAX,
            },
            UserRole::User => UserQuotas::default(),
            UserRole::ReadOnly => UserQuotas {
                max_channels: 0,
                max_assets: 0,
                max_transactions_per_day: 0,
                max_balance_btc: 0,
            },
        };

        self.set_user_quotas(user_id, &quotas).await?;
        self.set_user_settings(user_id, &UserSettings::default()).await?;

        Ok(())
    }

    /// Update user balance
    pub async fn update_user_balance(&self, user_id: &str, asset_id: Option<&str>, amount: i64) -> Result<(), AppError> {
        self.database.update_user_balance(user_id, asset_id, amount).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get user balance
    pub async fn get_user_balance(&self, user_id: &str, asset_id: Option<&str>) -> Result<i64, AppError> {
        self.database.get_user_balance(user_id, asset_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Save user transaction
    pub async fn save_user_transaction(&self, user_id: &str, txid: &str, amount: i64, asset_id: Option<&str>, status: &str) -> Result<(), AppError> {
        self.database.save_user_transaction(user_id, txid, amount, asset_id, status).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Save user address
    pub async fn save_user_address(&self, user_id: &str, address: &str) -> Result<(), AppError> {
        self.database.save_user_address(user_id, address).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get user transactions
    pub async fn get_user_transactions(&self, user_id: &str) -> Result<Vec<crate::database::UserTransaction>, AppError> {
        self.database.get_user_transactions(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Save user channel
    pub async fn save_user_channel(&self, user_id: &str, channel_id: &str, peer_pubkey: &str, capacity_sats: i64, status: &str) -> Result<(), AppError> {
        self.database.save_user_channel(user_id, channel_id, peer_pubkey, capacity_sats, status).await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get user channels
    pub async fn get_user_channels(&self, user_id: &str) -> Result<Vec<crate::database::UserChannel>, AppError> {
        self.database.get_user_channels(user_id).await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Check if user can perform action based on quotas
    pub async fn check_user_quota(&self, user_id: &str, action: &str) -> Result<bool, AppError> {
        let quotas = self.get_user_quotas(user_id).await?;
        
        match action {
            "create_channel" => {
                let current_channels = self.get_user_channels(user_id).await?.len() as u32;
                Ok(current_channels < quotas.max_channels)
            },
            "create_asset" => {
                // Would check current asset count
                Ok(true) // Simplified
            },
            "transaction" => {
                // Would check daily transaction count
                Ok(true) // Simplified
            },
            _ => Ok(true),
        }
    }

    /// Log user activity
    pub async fn log_activity(&self, activity: UserActivity) -> Result<(), AppError> {
        // Would implement activity logging to database
        tracing::info!("User activity: {} - {}", activity.user_id, activity.action);
        Ok(())
    }

    /// Get user quotas
    pub async fn get_user_quotas(&self, user_id: &str) -> Result<UserQuotas, AppError> {
        // Would implement database lookup
        Ok(UserQuotas::default())
    }

    /// Set user quotas
    pub async fn set_user_quotas(&self, user_id: &str, quotas: &UserQuotas) -> Result<(), AppError> {
        // Would implement database update
        tracing::info!("Updated quotas for user: {}", user_id);
        Ok(())
    }

    /// Get user settings
    pub async fn get_user_settings(&self, user_id: &str) -> Result<UserSettings, AppError> {
        // Would implement database lookup
        Ok(UserSettings::default())
    }

    /// Set user settings
    pub async fn set_user_settings(&self, user_id: &str, settings: &UserSettings) -> Result<(), AppError> {
        // Would implement database update
        tracing::info!("Updated settings for user: {}", user_id);
        Ok(())
    }
}

/// User permission checks
impl UserManager {
    pub fn can_read(&self, role: &UserRole) -> bool {
        matches!(role, UserRole::Admin | UserRole::User | UserRole::ReadOnly)
    }

    pub fn can_write(&self, role: &UserRole) -> bool {
        matches!(role, UserRole::Admin | UserRole::User)
    }

    pub fn can_admin(&self, role: &UserRole) -> bool {
        matches!(role, UserRole::Admin)
    }

    pub fn can_create_channels(&self, role: &UserRole) -> bool {
        matches!(role, UserRole::Admin | UserRole::User)
    }

    pub fn can_issue_assets(&self, role: &UserRole) -> bool {
        matches!(role, UserRole::Admin | UserRole::User)
    }
}