use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub session_token: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TelegramIntegration {
    users: Arc<RwLock<HashMap<i64, TelegramUser>>>,
    database_url: String,
}

impl TelegramIntegration {
    pub fn new(database_url: String) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            database_url,
        }
    }

    /// Register Telegram user for RGB Lightning Node access
    pub async fn register_user(&self, user_id: i64, username: Option<String>, first_name: String) -> Result<String, Box<dyn std::error::Error>> {
        let session_token = self.generate_session_token(user_id);
        
        let user = TelegramUser {
            user_id,
            username,
            first_name,
            session_token: session_token.clone(),
            created_at: Utc::now(),
        };

        // Store in memory cache
        self.users.write().await.insert(user_id, user.clone());

        // Create user directory for RGB data
        let user_dir = format!("dataldk0/users/{}", user_id);
        tokio::fs::create_dir_all(&user_dir).await?;

        Ok(session_token)
    }

    /// Validate user session
    pub async fn validate_session(&self, user_id: i64, token: &str) -> bool {
        if let Some(user) = self.users.read().await.get(&user_id) {
            user.session_token == token
        } else {
            false
        }
    }

    /// Get user RGB data directory
    pub fn get_user_data_dir(&self, user_id: i64) -> String {
        format!("dataldk0/users/{}", user_id)
    }

    fn generate_session_token(&self, user_id: i64) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(user_id.to_le_bytes());
        hasher.update(Utc::now().timestamp().to_le_bytes());
        hasher.update(b"rgb_lightning_session");
        format!("{:x}", hasher.finalize())[..32].to_string()
    }
}