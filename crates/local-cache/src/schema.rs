//! SQLite schema definitions for the local cache.
//!
//! This module defines the SQLite schema for caching document blobs locally
//! with support for LRU eviction and cloud synchronization tracking.
//!
//! # Schema
//!
//! The `local_cache` table stores metadata for cached document blobs:
//! - `id`: Primary key (UUID)
//! - `blob_path`: Path to local blob file (NULL if evicted)
//! - `size_bytes`: Size of the blob in bytes
//! - `cloud_version`: Version identifier from cloud storage
//! - `cloud_checksum`: SHA-256 checksum of the cloud blob
//! - `is_synced`: Whether the entry has been synced to cloud
//! - `evicted`: Whether the entry has been evicted
//! - `last_accessed_at`: Timestamp for LRU ordering
//!
//! # Indexes
//!
//! - `idx_local_cache_sync_lru`: Composite index on (is_synced, last_accessed_at)
//!   for efficient eviction candidate queries

use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use uuid::Uuid;

/// Name of the local cache table.
pub const TABLE_LOCAL_CACHE: &str = "local_cache";

/// SQL statement to create the local cache table.
const CREATE_LOCAL_CACHE_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS local_cache (
    id TEXT PRIMARY KEY,
    blob_path TEXT,
    size_bytes INTEGER NOT NULL,
    cloud_version INTEGER NOT NULL,
    cloud_checksum TEXT NOT NULL,
    is_synced INTEGER NOT NULL DEFAULT 0,
    evicted INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TEXT NOT NULL
);
"#;

/// SQL statement to create the eviction query index.
const CREATE_EVICTION_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_local_cache_sync_lru
ON local_cache (is_synced, last_accessed_at);
"#;

/// Initializes the SQLite schema for the local cache.
///
/// This function creates the `local_cache` table and the eviction index
/// if they don't already exist.
///
/// # Arguments
///
/// * `db` - SQLite connection pool
///
/// # Errors
///
/// Returns an error if the schema creation fails.
pub async fn initialize_schema(db: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create the local cache table
    sqlx::query(CREATE_LOCAL_CACHE_TABLE).execute(db).await?;

    // Create the eviction index for efficient LRU queries
    sqlx::query(CREATE_EVICTION_INDEX).execute(db).await?;

    Ok(())
}

/// A cache entry representing a cached document blob.
///
/// This struct maps to the `local_cache` table in SQLite.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct LocalCacheEntry {
    /// Unique identifier for this cache entry
    pub id: Uuid,
    /// Path to the local blob file (None if evicted)
    pub blob_path: Option<String>,
    /// Size of the blob in bytes
    pub size_bytes: i64,
    /// Cloud storage version for this entry
    pub cloud_version: i64,
    /// SHA-256 checksum of the cloud blob (hex encoded)
    pub cloud_checksum: String,
    /// Whether this entry has been synced to cloud storage
    pub is_synced: bool,
    /// Whether this entry has been evicted
    pub evicted: bool,
    /// Last access timestamp for LRU ordering
    pub last_accessed_at: chrono::DateTime<chrono::Utc>,
}

impl LocalCacheEntry {
    /// Creates a new cache entry with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier
    /// * `blob_path` - Path to the local blob file
    /// * `size_bytes` - Size of the blob in bytes
    /// * `cloud_version` - Cloud storage version
    /// * `cloud_checksum` - SHA-256 checksum (hex encoded)
    /// * `is_synced` - Whether synced to cloud
    /// * `evicted` - Whether evicted
    /// * `last_accessed_at` - Last access timestamp
    pub fn new(
        id: Uuid,
        blob_path: Option<String>,
        size_bytes: i64,
        cloud_version: i64,
        cloud_checksum: String,
        is_synced: bool,
        evicted: bool,
        last_accessed_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id,
            blob_path,
            size_bytes,
            cloud_version,
            cloud_checksum,
            is_synced,
            evicted,
            last_accessed_at,
        }
    }
}

/// Converts a SQLite row to a `LocalCacheEntry`.
///
/// # Arguments
///
/// * `row` - SQLite row to convert
///
/// # Returns
///
/// A `LocalCacheEntry` parsed from the row
///
/// # Errors
///
/// Returns an error if the row doesn't contain the expected columns.
impl TryFrom<SqliteRow> for LocalCacheEntry {
    type Error = sqlx::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let id: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let blob_path: Option<String> = row.try_get("blob_path")?;
        let size_bytes: i64 = row.try_get("size_bytes")?;
        let cloud_version: i64 = row.try_get("cloud_version")?;
        let cloud_checksum: String = row.try_get("cloud_checksum")?;
        let is_synced: i32 = row.try_get("is_synced")?;
        let evicted: i32 = row.try_get("evicted")?;
        let last_accessed_at: String = row.try_get("last_accessed_at")?;

        let last_accessed_at = chrono::DateTime::parse_from_rfc3339(&last_accessed_at)
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "last_accessed_at".to_string(),
                source: Box::new(e),
            })?
            .with_timezone(&chrono::Utc);

        Ok(Self {
            id,
            blob_path,
            size_bytes, // Keep as i64, not u64
            cloud_version,
            cloud_checksum,
            is_synced: is_synced != 0,
            evicted: evicted != 0,
            last_accessed_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    /// Creates an in-memory SQLite database for testing.
    async fn create_test_db() -> Result<SqlitePool, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .connect(":memory:")
            .await?;
        initialize_schema(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn test_schema_initialization() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;

        // Verify the table exists
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM local_cache")
            .fetch_one(&pool)
            .await?;
        assert_eq!(row.0, 0);

        // Verify the index exists
        let index_exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_local_cache_sync_lru'"
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(index_exists.0, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_insert_and_retrieve_entry() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;

        let entry = LocalCacheEntry::new(
            Uuid::new_v4(),
            Some("/path/to/blob".to_string()),
            1024,
            1,
            "abc123".to_string(),
            true,
            false,
            chrono::Utc::now(),
        );

        sqlx::query(
            r#"
            INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry.id.to_string())
        .bind(entry.blob_path.as_deref())
        .bind(entry.size_bytes as i64)
        .bind(entry.cloud_version)
        .bind(&entry.cloud_checksum)
        .bind(if entry.is_synced { 1i32 } else { 0i32 })
        .bind(if entry.evicted { 1i32 } else { 0i32 })
        .bind(entry.last_accessed_at.to_rfc3339())
        .execute(&pool)
        .await?;

        // Retrieve the entry
        let retrieved: LocalCacheEntry = sqlx::query_as(
            "SELECT id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at
             FROM local_cache WHERE id = ?"
        )
        .bind(entry.id.to_string())
        .fetch_one(&pool)
        .await?;

        assert_eq!(retrieved.id, entry.id);
        assert_eq!(retrieved.blob_path, entry.blob_path);
        assert_eq!(retrieved.size_bytes, entry.size_bytes);
        assert_eq!(retrieved.cloud_version, entry.cloud_version);
        assert_eq!(retrieved.cloud_checksum, entry.cloud_checksum);
        assert_eq!(retrieved.is_synced, entry.is_synced);
        assert_eq!(retrieved.evicted, entry.evicted);

        Ok(())
    }

    #[tokio::test]
    async fn test_eviction_candidate_query() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;

        // Insert entries with different sync statuses and access times
        let now = chrono::Utc::now();
        let older = now - chrono::Duration::seconds(60);
        let oldest = now - chrono::Duration::seconds(120);

        // Synced entry (can be evicted)
        sqlx::query(
            r#"
            INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind("/path/to/blob1".to_string())
        .bind(100i64)
        .bind(1i64)
        .bind("checksum1".to_string())
        .bind(1i32) // is_synced = true
        .bind(0i32) // evicted = false
        .bind(oldest.to_rfc3339())
        .execute(&pool)
        .await?;

        // Unsynced entry (cannot be evicted)
        sqlx::query(
            r#"
            INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind("/path/to/blob2".to_string())
        .bind(200i64)
        .bind(2i64)
        .bind("checksum2".to_string())
        .bind(0i32) // is_synced = false
        .bind(0i32) // evicted = false
        .bind(older.to_rfc3339())
        .execute(&pool)
        .await?;

        // Query for eviction candidates (synced, oldest first)
        let candidates: Vec<LocalCacheEntry> = sqlx::query_as(
            r#"
            SELECT id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at
            FROM local_cache
            WHERE is_synced = 1 AND evicted = 0
            ORDER BY last_accessed_at ASC
            LIMIT 10
            "#
        )
        .fetch_all(&pool)
        .await?;

        // Should only return the synced entry
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].blob_path, Some("/path/to/blob1".to_string()));

        Ok(())
    }
}