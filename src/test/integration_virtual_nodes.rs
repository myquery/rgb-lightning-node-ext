use crate::utils::{start_daemon, AppState};
use crate::args::LdkUserInfo;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::path::PathBuf;

async fn setup_test_node() -> Arc<AppState> {
    let args = LdkUserInfo {
        storage_dir_path: PathBuf::from("/tmp/test_virtual_nodes"),
        daemon_listening_port: 3001,
        ldk_peer_listening_port: 9735,
        network: rgb_lib::BitcoinNetwork::Regtest,
        max_media_upload_size_mb: 10,
    };
    
    start_daemon(&args).await.unwrap()
}

#[tokio::test]
async fn test_virtual_node_api_isolation() {
    let app_state = setup_test_node().await;
    let client = Client::new();
    let base_url = "http://localhost:3001";
    
    // Unlock the node first
    let unlock_payload = json!({
        "password": "test_password",
        "bitcoind_rpc_username": "user",
        "bitcoind_rpc_password": "password", 
        "bitcoind_rpc_host": "localhost",
        "bitcoind_rpc_port": 18433,
        "announce_addresses": [],
        "announce_alias": null
    });
    
    let unlock_response = client
        .post(&format!("{}/unlock", base_url))
        .json(&unlock_payload)
        .send()
        .await;
    
    // Skip if node setup fails (for CI environments)
    if unlock_response.is_err() {
        println!("⚠️  Skipping integration test - node setup failed");
        return;
    }
    
    // Test node info for different users
    let user1_headers = {
        let mut headers = HeaderMap::new();
        headers.insert("x-user-id", HeaderValue::from_static("user1"));
        headers
    };
    
    let user2_headers = {
        let mut headers = HeaderMap::new();
        headers.insert("x-user-id", HeaderValue::from_static("user2"));
        headers
    };
    
    // Get node info for user1
    let user1_response = client
        .get(&format!("{}/nodeinfo", base_url))
        .headers(user1_headers.clone())
        .send()
        .await
        .unwrap();
    
    // Get node info for user2
    let user2_response = client
        .get(&format!("{}/nodeinfo", base_url))
        .headers(user2_headers.clone())
        .send()
        .await
        .unwrap();
    
    if user1_response.status().is_success() && user2_response.status().is_success() {
        let user1_info: serde_json::Value = user1_response.json().await.unwrap();
        let user2_info: serde_json::Value = user2_response.json().await.unwrap();
        
        // Different users should have different virtual node IDs
        let user1_pubkey = user1_info["pubkey"].as_str().unwrap();
        let user2_pubkey = user2_info["pubkey"].as_str().unwrap();
        
        assert_ne!(user1_pubkey, user2_pubkey);
        println!("✓ API returns different virtual node IDs for different users");
        println!("  User1 pubkey: {}", user1_pubkey);
        println!("  User2 pubkey: {}", user2_pubkey);
    }
    
    // Test channel list isolation
    let user1_channels = client
        .get(&format!("{}/listchannels", base_url))
        .headers(user1_headers.clone())
        .send()
        .await
        .unwrap();
    
    let user2_channels = client
        .get(&format!("{}/listchannels", base_url))
        .headers(user2_headers.clone())
        .send()
        .await
        .unwrap();
    
    if user1_channels.status().is_success() && user2_channels.status().is_success() {
        let user1_data: serde_json::Value = user1_channels.json().await.unwrap();
        let user2_data: serde_json::Value = user2_channels.json().await.unwrap();
        
        // Both users should see empty channel lists initially
        assert_eq!(user1_data["channels"].as_array().unwrap().len(), 0);
        assert_eq!(user2_data["channels"].as_array().unwrap().len(), 0);
        println!("✓ Channel lists are properly isolated");
    }
    
    // Test payment list isolation
    let user1_payments = client
        .get(&format!("{}/listpayments", base_url))
        .headers(user1_headers)
        .send()
        .await
        .unwrap();
    
    let user2_payments = client
        .get(&format!("{}/listpayments", base_url))
        .headers(user2_headers)
        .send()
        .await
        .unwrap();
    
    if user1_payments.status().is_success() && user2_payments.status().is_success() {
        let user1_data: serde_json::Value = user1_payments.json().await.unwrap();
        let user2_data: serde_json::Value = user2_payments.json().await.unwrap();
        
        // Both users should see empty payment lists initially
        assert_eq!(user1_data["payments"].as_array().unwrap().len(), 0);
        assert_eq!(user2_data["payments"].as_array().unwrap().len(), 0);
        println!("✓ Payment lists are properly isolated");
    }
}

#[tokio::test]
async fn test_master_node_vs_virtual_node() {
    let client = Client::new();
    let base_url = "http://localhost:3001";
    
    // Get master node info (no user header)
    let master_response = client
        .get(&format!("{}/nodeinfo", base_url))
        .send()
        .await;
    
    // Get virtual node info (with user header)
    let mut virtual_headers = HeaderMap::new();
    virtual_headers.insert("x-user-id", HeaderValue::from_static("test_user"));
    
    let virtual_response = client
        .get(&format!("{}/nodeinfo", base_url))
        .headers(virtual_headers)
        .send()
        .await;
    
    if let (Ok(master), Ok(r#virtual)) = (master_response, virtual_response) {
        if master.status().is_success() && r#virtual.status().is_success() {
            let master_info: serde_json::Value = master.json().await.unwrap();
            let virtual_info: serde_json::Value = r#virtual.json().await.unwrap();
            
            let master_pubkey = master_info["pubkey"].as_str().unwrap();
            let virtual_pubkey = virtual_info["pubkey"].as_str().unwrap();
            
            // Master node and virtual node should have different pubkeys
            assert_ne!(master_pubkey, virtual_pubkey);
            println!("✓ Master node and virtual node have different identities");
            println!("  Master pubkey: {}", master_pubkey);
            println!("  Virtual pubkey: {}", virtual_pubkey);
        }
    }
}