use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct UserWallet {
    pub user_id: String,
    pub mnemonic_encrypted: String,
    pub derivation_path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct UserTransaction {
    pub id: Uuid,
    pub user_id: String,
    pub txid: String,
    pub amount: i64,
    pub asset_id: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct UserChannel {
    pub id: Uuid,
    pub user_id: String,
    pub channel_id: String,
    pub peer_pubkey: String,
    pub capacity_sats: i64,
    pub status: String,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        use sqlx::postgres::PgPoolOptions;
        
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .min_connections(0)
            .acquire_timeout(std::time::Duration::from_secs(1))
            .idle_timeout(std::time::Duration::from_secs(30))
            .max_lifetime(std::time::Duration::from_secs(300))
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // User wallet management
    pub async fn create_user_wallet(&self, user_id: &str, mnemonic_encrypted: &str, derivation_path: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO ln_user_wallets (user_id, mnemonic_encrypted, derivation_path) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO NOTHING",
            user_id, mnemonic_encrypted, derivation_path
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_user_wallet(&self, user_id: &str) -> Result<Option<UserWallet>> {
        let row = sqlx::query!(
            "SELECT user_id, mnemonic_encrypted, derivation_path, created_at FROM ln_user_wallets WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserWallet {
            user_id: r.user_id,
            mnemonic_encrypted: r.mnemonic_encrypted,
            derivation_path: r.derivation_path,
            created_at: r.created_at.unwrap_or_else(|| chrono::Utc::now()),
        }))
    }

    // User transaction tracking
    pub async fn save_user_transaction(&self, user_id: &str, txid: &str, amount: i64, asset_id: Option<&str>, status: &str) -> Result<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO ln_user_transactions (id, user_id, txid, amount, asset_id, status) VALUES ($1, $2, $3, $4, $5, $6)",
            id, user_id, txid, amount, asset_id, status
        )
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_user_transactions(&self, user_id: &str) -> Result<Vec<UserTransaction>> {
        let rows = sqlx::query!(
            "SELECT id, user_id, txid, amount, asset_id, status, created_at FROM ln_user_transactions WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| UserTransaction {
            id: r.id,
            user_id: r.user_id,
            txid: r.txid,
            amount: r.amount,
            asset_id: r.asset_id,
            status: r.status,
            created_at: r.created_at.unwrap_or_else(|| chrono::Utc::now()),
        }).collect())
    }

    // User channel management
    pub async fn save_user_channel(&self, user_id: &str, channel_id: &str, peer_pubkey: &str, capacity_sats: i64, status: &str) -> Result<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO ln_user_channels (id, user_id, channel_id, peer_pubkey, capacity_sats, status) VALUES ($1, $2, $3, $4, $5, $6)",
            id, user_id, channel_id, peer_pubkey, capacity_sats, status
        )
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_user_channels(&self, user_id: &str) -> Result<Vec<UserChannel>> {
        let rows = sqlx::query!(
            "SELECT id, user_id, channel_id, peer_pubkey, capacity_sats, status FROM ln_user_channels WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| UserChannel {
            id: r.id,
            user_id: r.user_id,
            channel_id: r.channel_id,
            peer_pubkey: r.peer_pubkey,
            capacity_sats: r.capacity_sats,
            status: r.status,
        }).collect())
    }

    // User balance tracking
    pub async fn update_user_balance(&self, user_id: &str, asset_id: Option<&str>, balance: i64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO ln_user_balances (user_id, asset_id, balance, updated_at) VALUES ($1, $2, $3, NOW()) 
             ON CONFLICT (user_id, COALESCE(asset_id, '')) DO UPDATE SET balance = $3, updated_at = NOW()",
            user_id, asset_id, balance
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_user_balance(&self, user_id: &str, asset_id: Option<&str>) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT balance FROM ln_user_balances WHERE user_id = $1 AND asset_id IS NOT DISTINCT FROM $2",
            user_id, asset_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.balance).unwrap_or(0))
    }

    pub async fn get_user_balances(&self, user_id: &str) -> Result<HashMap<String, i64>> {
        let rows = sqlx::query!(
            "SELECT asset_id, balance FROM ln_user_balances WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut balances = HashMap::new();
        for row in rows {
            let key = row.asset_id.unwrap_or_else(|| "BTC".to_string());
            balances.insert(key, row.balance);
        }
        Ok(balances)
    }

    // User address tracking
    pub async fn save_user_address(&self, user_id: &str, address: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO ln_user_addresses (user_id, address, created_at) VALUES ($1, $2, NOW()) ON CONFLICT (user_id, address) DO NOTHING",
            user_id, address
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_user_addresses(&self, user_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query!(
            "SELECT address FROM ln_user_addresses WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.address).collect())
    }

    /// Save virtual node ID for user
    pub async fn save_virtual_node_id(&self, user_id: &str, virtual_node_id: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE ln_user_wallets SET virtual_node_id = $2 WHERE user_id = $1",
            user_id, virtual_node_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get virtual node ID for user
    pub async fn get_virtual_node_id(&self, user_id: &str) -> Result<Option<String>> {
        let row = sqlx::query!(
            "SELECT virtual_node_id FROM ln_user_wallets WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.and_then(|r| r.virtual_node_id))
    }
}