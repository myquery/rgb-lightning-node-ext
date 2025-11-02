use crate::database::Database;
use uuid::Uuid;

/// Virtual channel mapping for multi-user support
#[derive(Clone)]
pub struct VirtualChannelManager {
    database: Database,
}

impl VirtualChannelManager {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Map channel to virtual node
    pub async fn map_channel_to_virtual_node(&self, channel_id: &str, virtual_node_id: &str, user_id: i64) -> Result<(), sqlx::Error> {
        // Use existing ln_user_channels table instead of virtual_channels
        sqlx::query!(
            "INSERT INTO ln_user_channels (id, user_id, channel_id, peer_pubkey, capacity_sats, status) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (channel_id) DO NOTHING",
            Uuid::new_v4(), user_id.to_string(), channel_id, virtual_node_id, 0i64, "mapped"
        )
        .execute(self.database.pool())
        .await?;
        Ok(())
    }

    /// Get channels for virtual node
    pub async fn get_channels_for_virtual_node(&self, virtual_node_id: &str) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT channel_id FROM ln_user_channels WHERE peer_pubkey = $1",
            virtual_node_id
        )
        .fetch_all(self.database.pool())
        .await?;
        
        Ok(rows.into_iter().map(|r| r.channel_id).collect())
    }

    /// Get virtual node for channel
    pub async fn get_virtual_node_for_channel(&self, channel_id: &str) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT peer_pubkey FROM ln_user_channels WHERE channel_id = $1",
            channel_id
        )
        .fetch_optional(self.database.pool())
        .await?;
        
        Ok(row.map(|r| r.peer_pubkey))
    }

    /// Map payment to virtual node
    pub async fn map_payment_to_virtual_node(&self, payment_hash: &str, _virtual_node_id: &str, user_id: i64, inbound: bool) -> Result<(), sqlx::Error> {
        // Use existing ln_user_transactions table instead of virtual_payments
        let amount = if inbound { 1000 } else { -1000 }; // Placeholder amount
        sqlx::query!(
            "INSERT INTO ln_user_transactions (id, user_id, txid, amount, asset_id, status) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (txid) DO NOTHING",
            Uuid::new_v4(), user_id.to_string(), payment_hash, amount, None::<String>, "mapped"
        )
        .execute(self.database.pool())
        .await?;
        Ok(())
    }

    /// Get payments for virtual node
    pub async fn get_payments_for_virtual_node(&self, _virtual_node_id: &str) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT txid FROM ln_user_transactions WHERE status = 'mapped'",
        )
        .fetch_all(self.database.pool())
        .await?;
        
        Ok(rows.into_iter().map(|r| r.txid).collect())
    }
}