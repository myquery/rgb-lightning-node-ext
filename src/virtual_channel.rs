use crate::database::Database;
use bitcoin::secp256k1::PublicKey;
use lightning::ln::types::ChannelId;
use std::collections::HashMap;

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
        sqlx::query!(
            "INSERT INTO virtual_channels (channel_id, virtual_node_id, user1_id, created_at) VALUES ($1, $2, $3, NOW()) ON CONFLICT (channel_id) DO UPDATE SET virtual_node_id = $2, user1_id = $3",
            channel_id, virtual_node_id, user_id
        )
        .execute(self.database.pool())
        .await?;
        Ok(())
    }

    /// Get channels for virtual node
    pub async fn get_channels_for_virtual_node(&self, virtual_node_id: &str) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT channel_id FROM virtual_channels WHERE virtual_node_id = $1",
            virtual_node_id
        )
        .fetch_all(self.database.pool())
        .await?;
        
        Ok(rows.into_iter().map(|r| r.channel_id).collect())
    }

    /// Get virtual node for channel
    pub async fn get_virtual_node_for_channel(&self, channel_id: &str) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT virtual_node_id FROM virtual_channels WHERE channel_id = $1",
            channel_id
        )
        .fetch_optional(self.database.pool())
        .await?;
        
        Ok(row.and_then(|r| r.virtual_node_id))
    }

    /// Map payment to virtual node
    pub async fn map_payment_to_virtual_node(&self, payment_hash: &str, virtual_node_id: &str, user_id: i64, inbound: bool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO virtual_payments (payment_hash, virtual_node_id, user_id, inbound, created_at) VALUES ($1, $2, $3, $4, NOW()) ON CONFLICT (payment_hash) DO UPDATE SET virtual_node_id = $2, user_id = $3",
            payment_hash, virtual_node_id, &user_id.to_string(), inbound
        )
        .execute(self.database.pool())
        .await?;
        Ok(())
    }

    /// Get payments for virtual node
    pub async fn get_payments_for_virtual_node(&self, virtual_node_id: &str) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT payment_hash FROM virtual_payments WHERE virtual_node_id = $1",
            virtual_node_id
        )
        .fetch_all(self.database.pool())
        .await?;
        
        Ok(rows.into_iter().map(|r| r.payment_hash).collect())
    }
}