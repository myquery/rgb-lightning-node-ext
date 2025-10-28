use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RgbDatabaseAdapter {
    pool: Arc<PgPool>,
    sqlite_to_pg_map: Arc<Mutex<HashMap<String, String>>>,
}

impl RgbDatabaseAdapter {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            sqlite_to_pg_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Intercept SQLite database path and redirect to PostgreSQL
    pub async fn setup_rgb_database_redirect(&self) -> Result<(), sqlx::Error> {
        // Set environment variable to redirect RGB library database calls
        let pg_url = env::var("DATABASE_URL").unwrap_or_default();
        env::set_var("RGB_DATABASE_URL", pg_url);
        env::set_var("RGB_USE_POSTGRESQL", "true");
        
        // Run RGB table migrations
        self.migrate_rgb_tables().await?;
        
        Ok(())
    }

    async fn migrate_rgb_tables(&self) -> Result<(), sqlx::Error> {
        // Check if RGB tables exist, create if not
        let exists = sqlx::query("SELECT 1 FROM information_schema.tables WHERE table_name = 'rgb_txo'")
            .fetch_optional(&*self.pool)
            .await?;

        if exists.is_none() {
            // Run RGB table creation migration
            let migration_sql = include_str!("../migrations/20250826000020_rgb_tables.sql");
            sqlx::raw_sql(migration_sql).execute(&*self.pool).await?;
        }

        Ok(())
    }

    // Convert SQLite queries to PostgreSQL
    pub fn convert_sqlite_to_postgres(&self, query: &str) -> String {
        query
            .replace("AUTOINCREMENT", "")
            .replace("INTEGER PRIMARY KEY", "SERIAL PRIMARY KEY")
            .replace("json_text", "JSONB")
            .replace("tinyint", "SMALLINT")
            .replace("\"txo\"", "\"rgb_txo\"")
            .replace("\"media\"", "\"rgb_media\"")
            .replace("\"asset\"", "\"rgb_asset\"")
            .replace("\"batch_transfer\"", "\"rgb_batch_transfer\"")
            .replace("\"asset_transfer\"", "\"rgb_asset_transfer\"")
            .replace("\"coloring\"", "\"rgb_coloring\"")
            .replace("\"transfer\"", "\"rgb_transfer\"")
            .replace("\"transport_endpoint\"", "\"rgb_transport_endpoint\"")
            .replace("\"transfer_transport_endpoint\"", "\"rgb_transfer_transport_endpoint\"")
            .replace("\"token\"", "\"rgb_token\"")
            .replace("\"token_media\"", "\"rgb_token_media\"")
            .replace("\"wallet_transaction\"", "\"rgb_wallet_transaction\"")
            .replace("\"pending_witness_script\"", "\"rgb_pending_witness_script\"")
            .replace("\"backup_info\"", "\"rgb_backup_info\"")
            .replace("\"seaql_migrations\"", "\"rgb_seaql_migrations\"")
    }
}

// Hook into RGB library's database initialization
pub fn setup_rgb_postgres_adapter(pool: Arc<PgPool>) -> RgbDatabaseAdapter {
    let adapter = RgbDatabaseAdapter::new(pool);
    
    // Set environment variables to redirect RGB library to use our adapter
    env::set_var("RGB_DATABASE_TYPE", "postgresql");
    env::set_var("RGB_DISABLE_SQLITE", "true");
    
    adapter
}