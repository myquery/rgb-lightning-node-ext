use std::env;
use std::sync::Once;
use std::time::Duration;
use tokio::time::sleep;

static INIT: Once = Once::new();

pub fn configure_rgb_database() {
    INIT.call_once(|| {
        // Force RGB library to use in-memory SQLite to avoid connection pool conflicts
        env::set_var("SQLITE_THREADSAFE", "0"); // Single-threaded mode
        env::set_var("SQLITE_CACHE_SIZE", "2000");
        env::set_var("SQLITE_TEMP_STORE", "2"); // Memory temp storage
        env::set_var("SQLITE_SYNCHRONOUS", "0"); // No sync for speed
        env::set_var("SQLITE_JOURNAL_MODE", "MEMORY"); // Memory journal
        env::set_var("SQLITE_LOCKING_MODE", "EXCLUSIVE");
        env::set_var("SQLITE_BUSY_TIMEOUT", "0"); // No waiting
        env::set_var("SQLX_OFFLINE", "true");
        
        // Try to redirect RGB database to memory if possible
        if let Err(e) = crate::sqlite_proxy::redirect_rgb_database_to_memory() {
            tracing::warn!("Could not redirect RGB database to memory: {}", e);
        }
    });
}

// Force RGB library to use single-threaded database access
pub async fn wait_for_rgb_ready() {
    // Give RGB library time to initialize properly
    sleep(Duration::from_secs(2)).await;
}