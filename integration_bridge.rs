use serde_json::Value;
use std::collections::HashMap;

/// Bridge service connecting bitMaskRGB bot with RGB Lightning Node
pub struct RgbLightningBridge {
    node_url: String,
    client: reqwest::Client,
}

impl RgbLightningBridge {
    pub fn new(node_url: &str) -> Self {
        Self {
            node_url: node_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Proxy RGB asset balance from Lightning Node
    pub async fn get_user_rgb_balance(&self, user_id: i64, asset_id: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&format!("{}/assetbalance", self.node_url))
            .json(&serde_json::json!({
                "user_id": user_id,
                "asset_id": asset_id
            }))
            .send()
            .await?;
        
        let balance: Value = response.json().await?;
        Ok(balance["balance"].as_u64().unwrap_or(0))
    }

    /// Create RGB invoice via Lightning Node
    pub async fn create_rgb_invoice(&self, user_id: i64, asset_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&format!("{}/rgbinvoice", self.node_url))
            .json(&serde_json::json!({
                "user_id": user_id,
                "asset_id": asset_id
            }))
            .send()
            .await?;
        
        let invoice: Value = response.json().await?;
        Ok(invoice["invoice"].as_str().unwrap_or("").to_string())
    }

    /// Send RGB tokens via Lightning Node
    pub async fn send_rgb_tokens(&self, user_id: i64, invoice: &str, asset_id: &str, amount: u64) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&format!("{}/sendasset", self.node_url))
            .json(&serde_json::json!({
                "user_id": user_id,
                "invoice": invoice,
                "asset_id": asset_id,
                "amount": amount
            }))
            .send()
            .await?;
        
        let result: Value = response.json().await?;
        Ok(result["txid"].as_str().unwrap_or("").to_string())
    }
}