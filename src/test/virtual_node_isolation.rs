use crate::hsm_provider::{HsmProvider, LocalHsmProvider};
use crate::virtual_context::VirtualNodeContext;
use crate::virtual_channel::VirtualChannelManager;
use crate::database::Database;
use crate::user_manager::UserManager;
use bitcoin::secp256k1::PublicKey;
use lightning::sign::KeysManager;
use std::sync::Arc;
use std::str::FromStr;

#[tokio::test]
async fn test_virtual_node_key_derivation() {
    // Create mock keys manager
    let keys_manager = Arc::new(KeysManager::new(
        &[1u8; 32],
        42,
        42,
        std::path::PathBuf::from("/tmp"),
    ));
    
    let hsm_provider = Arc::new(LocalHsmProvider::new(keys_manager));
    
    // Test deterministic key derivation
    let user1_node_id = hsm_provider.derive_virtual_node_id("user1").await.unwrap();
    let user2_node_id = hsm_provider.derive_virtual_node_id("user2").await.unwrap();
    let user1_node_id_again = hsm_provider.derive_virtual_node_id("user1").await.unwrap();
    
    // Same user should get same virtual node ID
    assert_eq!(user1_node_id, user1_node_id_again);
    
    // Different users should get different virtual node IDs
    assert_ne!(user1_node_id, user2_node_id);
    
    println!("✓ Virtual node key derivation works correctly");
}

#[tokio::test]
async fn test_virtual_channel_isolation() {
    // Setup test database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost:5432/test_rgb_lightning".to_string());
    
    let database = Database::new(&database_url).await.unwrap();
    let channel_manager = VirtualChannelManager::new(database);
    
    // Create test data
    let channel1 = "channel_001";
    let channel2 = "channel_002";
    let user1_virtual_node = "03abc123";
    let user2_virtual_node = "03def456";
    
    // Map channels to different virtual nodes
    channel_manager.map_channel_to_virtual_node(channel1, user1_virtual_node, "user1").await.unwrap();
    channel_manager.map_channel_to_virtual_node(channel2, user2_virtual_node, "user2").await.unwrap();
    
    // Test isolation
    let user1_channels = channel_manager.get_channels_for_virtual_node(user1_virtual_node).await.unwrap();
    let user2_channels = channel_manager.get_channels_for_virtual_node(user2_virtual_node).await.unwrap();
    
    assert_eq!(user1_channels, vec![channel1]);
    assert_eq!(user2_channels, vec![channel2]);
    assert!(!user1_channels.contains(&channel2.to_string()));
    assert!(!user2_channels.contains(&channel1.to_string()));
    
    println!("✓ Virtual channel isolation works correctly");
}

#[tokio::test]
async fn test_virtual_payment_isolation() {
    // Setup test database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost:5432/test_rgb_lightning".to_string());
    
    let database = Database::new(&database_url).await.unwrap();
    let channel_manager = VirtualChannelManager::new(database);
    
    // Create test data
    let payment1 = "payment_hash_001";
    let payment2 = "payment_hash_002";
    let user1_virtual_node = "03abc123";
    let user2_virtual_node = "03def456";
    
    // Map payments to different virtual nodes
    channel_manager.map_payment_to_virtual_node(payment1, user1_virtual_node, "user1", true).await.unwrap();
    channel_manager.map_payment_to_virtual_node(payment2, user2_virtual_node, "user2", false).await.unwrap();
    
    // Test isolation
    let user1_payments = channel_manager.get_payments_for_virtual_node(user1_virtual_node).await.unwrap();
    let user2_payments = channel_manager.get_payments_for_virtual_node(user2_virtual_node).await.unwrap();
    
    assert_eq!(user1_payments, vec![payment1]);
    assert_eq!(user2_payments, vec![payment2]);
    assert!(!user1_payments.contains(&payment2.to_string()));
    assert!(!user2_payments.contains(&payment1.to_string()));
    
    println!("✓ Virtual payment isolation works correctly");
}

#[tokio::test]
async fn test_virtual_node_context_extraction() {
    // Test user ID extraction from different sources
    let headers = axum::http::HeaderMap::new();
    
    // Test extraction from JSON body
    let payload_with_user = serde_json::json!({"user_id": "test_user"});
    let user_id = crate::user_manager::UserManager::extract_user_id(&payload_with_user, &headers);
    assert_eq!(user_id, Some("test_user".to_string()));
    
    // Test extraction failure
    let payload_without_user = serde_json::json!({"other_field": "value"});
    let no_user_id = crate::user_manager::UserManager::extract_user_id(&payload_without_user, &headers);
    assert_eq!(no_user_id, None);
    
    println!("✓ Virtual node context extraction works correctly");
}

#[tokio::test]
async fn test_cross_user_data_leakage() {
    // Setup test database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost:5432/test_rgb_lightning".to_string());
    
    let database = Database::new(&database_url).await.unwrap();
    let channel_manager = VirtualChannelManager::new(database);
    
    // Create test data for multiple users
    let users = vec![
        ("user1", "03abc111", "channel_001", "payment_001"),
        ("user2", "03abc222", "channel_002", "payment_002"),
        ("user3", "03abc333", "channel_003", "payment_003"),
    ];
    
    // Map data to virtual nodes
    for (user_id, virtual_node_id, channel_id, payment_hash) in &users {
        channel_manager.map_channel_to_virtual_node(channel_id, virtual_node_id, user_id).await.unwrap();
        channel_manager.map_payment_to_virtual_node(payment_hash, virtual_node_id, user_id, true).await.unwrap();
    }
    
    // Verify each user only sees their own data
    for (user_id, virtual_node_id, expected_channel, expected_payment) in &users {
        let channels = channel_manager.get_channels_for_virtual_node(virtual_node_id).await.unwrap();
        let payments = channel_manager.get_payments_for_virtual_node(virtual_node_id).await.unwrap();
        
        // Should only see own data
        assert_eq!(channels.len(), 1);
        assert_eq!(payments.len(), 1);
        assert_eq!(channels[0], *expected_channel);
        assert_eq!(payments[0], *expected_payment);
        
        // Should not see other users' data
        for (other_user, _, other_channel, other_payment) in &users {
            if other_user != user_id {
                assert!(!channels.contains(&other_channel.to_string()));
                assert!(!payments.contains(&other_payment.to_string()));
            }
        }
    }
    
    println!("✓ No cross-user data leakage detected");
}

#[tokio::test]
async fn test_virtual_node_id_consistency() {
    // Create multiple HSM providers with same keys
    let keys_manager = Arc::new(KeysManager::new(
        &[42u8; 32],
        123,
        456,
        std::path::PathBuf::from("/tmp"),
    ));
    
    let hsm1 = Arc::new(LocalHsmProvider::new(keys_manager.clone()));
    let hsm2 = Arc::new(LocalHsmProvider::new(keys_manager.clone()));
    
    // Same user should get same virtual node ID across different HSM instances
    let user_id = "consistency_test_user";
    let node_id1 = hsm1.derive_virtual_node_id(user_id).await.unwrap();
    let node_id2 = hsm2.derive_virtual_node_id(user_id).await.unwrap();
    
    assert_eq!(node_id1, node_id2);
    
    // Different seed should produce different virtual node ID
    let different_keys = Arc::new(KeysManager::new(
        &[99u8; 32],
        123,
        456,
        std::path::PathBuf::from("/tmp"),
    ));
    let hsm3 = Arc::new(LocalHsmProvider::new(different_keys));
    let node_id3 = hsm3.derive_virtual_node_id(user_id).await.unwrap();
    
    assert_ne!(node_id1, node_id3);
    
    println!("✓ Virtual node ID consistency verified");
}