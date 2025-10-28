use crate::auth::{Claims, UserRole, AuthService};
use crate::error::APIError;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramUser {
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub is_bot: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramAuthData {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: i64,
    pub hash: String,
}

pub struct TelegramAuthService {
    bot_token: String,
    auth_service: AuthService,
}

impl TelegramAuthService {
    pub fn new(bot_token: String, auth_service: AuthService) -> Self {
        Self { bot_token, auth_service }
    }

    /// Verify Telegram auth data and generate JWT token
    pub fn authenticate_telegram_user(&self, auth_data: TelegramAuthData) -> Result<String, APIError> {
        // Verify Telegram auth hash
        if !self.verify_telegram_auth(&auth_data) {
            return Err(APIError::Unauthorized("Invalid Telegram authentication".to_string()));
        }

        // Create user_id from telegram_id
        let user_id = format!("tg_{}", auth_data.id);
        
        // Determine role (default to User for Telegram users)
        let role = UserRole::User;

        // Generate JWT token
        self.auth_service.generate_token(user_id, role)
    }

    /// Extract Telegram user from headers (for bot service calls)
    pub fn extract_telegram_user_from_headers(headers: &HeaderMap) -> Option<String> {
        // Check for Telegram user ID in custom header
        if let Some(tg_user_id) = headers.get("x-telegram-user-id").and_then(|v| v.to_str().ok()) {
            return Some(format!("tg_{}", tg_user_id));
        }

        // Check for bot token authentication
        if let Some(bot_token) = headers.get("x-bot-token").and_then(|v| v.to_str().ok()) {
            if bot_token == std::env::var("BOT_TOKEN").unwrap_or_default() {
                // Extract user from bot request body would be handled in middleware
                return None;
            }
        }

        None
    }

    /// Verify Telegram authentication data
    fn verify_telegram_auth(&self, auth_data: &TelegramAuthData) -> bool {
        use sha2::{Sha256, Digest};
        use hmac::{Hmac, Mac};

        // Create data check string
        let mut check_data = HashMap::new();
        check_data.insert("id", auth_data.id.to_string());
        check_data.insert("first_name", auth_data.first_name.clone());
        if let Some(last_name) = &auth_data.last_name {
            check_data.insert("last_name", last_name.clone());
        }
        if let Some(username) = &auth_data.username {
            check_data.insert("username", username.clone());
        }
        if let Some(photo_url) = &auth_data.photo_url {
            check_data.insert("photo_url", photo_url.clone());
        }
        check_data.insert("auth_date", auth_data.auth_date.to_string());

        // Sort and create check string
        let mut sorted_keys: Vec<_> = check_data.keys().collect();
        sorted_keys.sort();
        let data_check_string = sorted_keys
            .iter()
            .map(|k| format!("{}={}", k, check_data[*k]))
            .collect::<Vec<_>>()
            .join("\n");

        // Create secret key
        let mut hasher = Sha256::new();
        hasher.update(self.bot_token.as_bytes());
        let secret_key = hasher.finalize();

        // Create HMAC
        let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key).unwrap();
        mac.update(data_check_string.as_bytes());
        let result = mac.finalize();
        let calculated_hash = hex::encode(result.into_bytes());

        calculated_hash == auth_data.hash
    }

    /// Create user context for Telegram user
    pub fn create_telegram_user_context(telegram_id: i64, username: Option<String>) -> Claims {
        Claims {
            user_id: format!("tg_{}", telegram_id),
            role: UserRole::User,
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        }
    }
}

/// Middleware for Telegram bot authentication
pub async fn telegram_bot_auth_middleware(
    headers: HeaderMap,
    body: &serde_json::Value,
) -> Result<Option<Claims>, APIError> {
    // Check for bot token
    if let Some(bot_token) = headers.get("x-bot-token").and_then(|v| v.to_str().ok()) {
        let expected_token = std::env::var("BOT_TOKEN")
            .map_err(|_| APIError::Unauthorized("Bot token not configured".to_string()))?;
        
        if bot_token != expected_token {
            return Err(APIError::Unauthorized("Invalid bot token".to_string()));
        }

        // Extract Telegram user from request body
        if let Some(telegram_id) = body.get("telegram_user_id").and_then(|v| v.as_i64()) {
            let username = body.get("telegram_username").and_then(|v| v.as_str()).map(|s| s.to_string());
            
            return Ok(Some(TelegramAuthService::create_telegram_user_context(
                telegram_id, 
                username
            )));
        }
    }

    // Check for direct Telegram user header
    if let Some(user_id) = TelegramAuthService::extract_telegram_user_from_headers(&headers) {
        return Ok(Some(Claims {
            user_id,
            role: UserRole::User,
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        }));
    }

    Ok(None)
}