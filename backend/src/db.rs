//! Database connection and initialization

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

pub type DbPool = SqlitePool;

/// Initialize database connection with optimizations for 500 MB RAM
pub async fn init_db(database_url: &str) -> Result<DbPool, String> {
    tracing::info!("🗄️  Connecting to SQLite database (embedded)");

    // Extract database path from URL
    let db_path = database_url
        .strip_prefix("sqlite:")
        .ok_or_else(|| "Invalid SQLite URL".to_string())?;

    // Create data directory if it doesn't exist
    if let Some(parent) = Path::new(db_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
            tracing::info!("📁 Created data directory: {}", parent.display());
        }
    }

    // Create database file if it doesn't exist
    if !Path::new(db_path).exists() {
        std::fs::File::create(db_path)
            .map_err(|e| format!("Failed to create database file: {}", e))?;
        tracing::info!("📄 Created database file: {}", db_path);
    }

    // Create connection pool with single connection (single-threaded)
    let pool = SqlitePoolOptions::new()
        .max_connections(1) // Single-threaded, one connection is enough
        .connect(database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    tracing::info!("Testing database connection...");
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Database connection test failed: {}", e))?;

    tracing::info!("✅ SQLite database connected successfully!");

    // Apply PRAGMA optimizations for 500 MB RAM constraint
    tracing::info!("⚙️  Applying PRAGMA optimizations...");
    
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to set journal_mode: {}", e))?;
    
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to set synchronous: {}", e))?;
    
    sqlx::query("PRAGMA cache_size = -10000")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to set cache_size: {}", e))?;
    
    sqlx::query("PRAGMA temp_store = MEMORY")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to set temp_store: {}", e))?;
    
    sqlx::query("PRAGMA mmap_size = 0")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to set mmap_size: {}", e))?;
    
    sqlx::query("PRAGMA page_size = 4096")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to set page_size: {}", e))?;
    
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to enable foreign_keys: {}", e))?;

    tracing::info!("✅ PRAGMA optimizations applied");

    // Apply schema and create tables
    tracing::info!("📋 Applying database schema...");
    
    // Create cards table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            total_lines INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to create cards table: {}", e))?;
    
    // Create card_chunks table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS card_chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id TEXT NOT NULL,
            line_number INTEGER NOT NULL,
            line_text TEXT NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to create card_chunks table: {}", e))?;
    
    // Create index
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_card_chunks_lookup 
        ON card_chunks(card_id, line_number)
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Failed to create index: {}", e))?;
    
    tracing::info!("✅ Database schema applied successfully");

    // Run migrations
    tracing::info!("🔄 Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    tracing::info!("✅ Database migrations completed");

    Ok(pool)
}
