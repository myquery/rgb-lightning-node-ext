use std::sync::Arc;
use crate::blockchain_balance::BlockchainBalanceService;
use crate::bitcoind::BitcoindClient;
use crate::disk::FilesystemLogger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "tb1p2l333xrmymmx6jhur5ku0rw2sltdp774mthltguwt8hwsj8utxzsgx3st9";
    
    // Initialize BitcoindClient (testnet)
    let logger = Arc::new(FilesystemLogger::new("./logs".into()));
    let bitcoind_client = BitcoindClient::new(
        "electrum.iriswallet.com".to_string(),
        18332,
        "user".to_string(),
        "password".to_string(),
        tokio::runtime::Handle::current(),
        logger,
    ).await?;
    
    // Create blockchain balance service
    let blockchain_service = BlockchainBalanceService::new(Arc::new(bitcoind_client));
    
    // Query balance
    match blockchain_service.get_address_balance(address).await {
        Ok(balance) => {
            println!("Address: {}", address);
            println!("Balance: {} satoshis", balance);
            println!("Balance: {:.8} BTC", balance as f64 / 100_000_000.0);
        }
        Err(e) => {
            println!("Error querying balance: {}", e);
        }
    }
    
    Ok(())
}