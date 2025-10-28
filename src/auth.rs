use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{error::APIError, utils::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub role: UserRole,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserRole {
    Admin,
    User,
    ReadOnly,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_hours: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string()),
            token_expiry_hours: 24,
        }
    }
}

pub struct AuthService {
    config: AuthConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        
        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }

    pub fn generate_token(&self, user_id: String, role: UserRole) -> Result<String, APIError> {
        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(self.config.token_expiry_hours as i64))
            .unwrap()
            .timestamp() as usize;

        let claims = Claims { user_id, role, exp };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| APIError::Unexpected(format!("Token generation failed: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, APIError> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| APIError::Unauthorized(format!("Invalid token: {}", e)))
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    use crate::telegram_auth::telegram_bot_auth_middleware;
    
    // Skip auth for certain endpoints
    let path = request.uri().path();
    if matches!(path, "/init" | "/unlock" | "/networkinfo") {
        return Ok(next.run(request).await);
    }

    // Try Telegram bot authentication first
    let body_bytes = axum::body::to_bytes(request.body_mut(), usize::MAX).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body: serde_json::Value = serde_json::from_slice(&body_bytes)
        .unwrap_or(serde_json::Value::Null);
    
    if let Ok(Some(claims)) = telegram_bot_auth_middleware(headers.clone(), &body).await {
        request.extensions_mut().insert(claims);
        return Ok(next.run(request).await);
    }

    // Fallback to JWT token authentication
    let token = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_service = AuthService::new(AuthConfig::default());
    let claims = auth_service.validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub role: UserRole,
}