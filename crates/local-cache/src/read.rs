//! Cache read operations for the local cache module.
//!
//! This module provides functions for reading cached document blobs
//! with automatic last_accessed_at updates and cloud fallback.
//!
//! # Performance Requirements
//!
//! - Cache reads MUST complete within 10ms at p95 (Requirement 2.1, 9.1)
//! - last_accessed_at MUST be updated on every read
//! - Cloud fetch is triggered when cache miss occurs

use sqlx::{SqlitePool}; // Row removed - unused
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::schema::{LocalCacheEntry}; // TABLE_LOCAL_CACHE removed - unused
use shared::error::CacheError;

/// Result type for cache read operations.
pub type CacheReadResult = Result<CacheRead, CacheError>;

/// Represents the result of a cache read operation.
#[derive(Debug, Clone)]
pub struct CacheRead {
    /// The cached blob data
    pub data: Vec<u8>,
    /// The cache entry metadata
    pub entry: LocalCacheEntry,
    /// Whether the data was served from cache (false = fetched from cloud)
    pub from_cache: bool,
}

/// Fetches a document blob from cache or cloud storage.
///
/// This function first checks the local cache. If the entry exists and is not
/// evicted, it returns the cached blob and updates last_accessed_at. If the
/// entry is not found in cache, it fetches from cloud storage.
///
/// # Arguments
///
/// * `db` - SQLite connection pool
/// * `entry_id` - UUID of the cache entry to retrieve
/// * `cloud_fetcher` - Async function to fetch blob from cloud storage
///
/// # Returns
///
/// Returns `CacheReadResult` containing the blob data and metadata.
///
/// # Errors
///
/// Returns `CacheError::EntryNotFound` if the entry doesn't exist.
/// Returns `CacheError::EntryEvicted` if the entry was evicted.
/// Returns errors from cloud fetcher if cache miss and cloud fetch fails.
pub async fn get_or_fetch_blob<C, F>(
    db: &SqlitePool,
    entry_id: Uuid,
    mut cloud_fetcher: F,
) -> CacheReadResult
where
    F: FnMut(Uuid) -> C,
    C: std::future::Future<Output = Result<(Vec<u8>, i64, String), Box<dyn std::error::Error + Send + Sync>>>,
{
    // Try to get from cache first
    if let Some(entry) = get_cache_entry(db, entry_id).await? {
        // Check if entry is available (not evicted and has blob_path)
        if entry.evicted || entry.blob_path.is_none() {
            // Entry is evicted, need to fetch from cloud
            tracing::debug!(entry_id = %entry_id, "Cache miss - entry evicted");
            return fetch_and_cache(db, entry_id, &mut cloud_fetcher).await;
        }

        // Read the blob from local filesystem
        if let Some(blob_path) = &entry.blob_path {
            match read_blob_from_disk(blob_path).await {
                Ok(data) => {
                    // Update last_accessed_at timestamp
                    update_last_accessed(db, entry_id).await?;

                    tracing::debug!(
                        entry_id = %entry_id,
                        size_bytes = data.len(),
                        "Cache hit - served from local cache"
                    );

                    return Ok(CacheRead {
                        data,
                        entry,
                        from_cache: true,
                    });
                }
                Err(e) => {
                    tracing::warn!(entry_id = %entry_id, error = %e, "Failed to read blob from disk");
                    // Fall through to cloud fetch
                }
            }
        }
    }

    // Cache miss - fetch from cloud
    tracing::debug!(entry_id = %entry_id, "Cache miss - fetching from cloud");
    fetch_and_cache(db, entry_id, &mut cloud_fetcher).await
}

/// Fetches a blob from cloud storage and caches it locally.
///
/// # Arguments
///
/// * `db` - SQLite connection pool
/// * `entry_id` - UUID of the cache entry
/// * `cloud_fetcher` - Async function to fetch blob from cloud storage
///
/// # Returns
///
/// Returns `CacheReadResult` containing the fetched blob data.
async fn fetch_and_cache<C, F>(
    db: &SqlitePool,
    entry_id: Uuid,
    cloud_fetcher: &mut F,
) -> CacheReadResult
where
    F: FnMut(Uuid) -> C,
    C: std::future::Future<Output = Result<(Vec<u8>, i64, String), Box<dyn std::error::Error + Send + Sync>>>,
{
    // Fetch from cloud
    let (data, cloud_version, cloud_checksum) = cloud_fetcher(entry_id)
        .await
        .map_err(|e| CacheError::CloudVerificationFailed {
            id: entry_id,
            reason: format!("Cloud fetch failed: {}", e),
        })?;

    // Create cache directory if it doesn't exist
    let cache_dir = get_cache_directory().await?;
    fs::create_dir_all(&cache_dir).await.map_err(|e| CacheError::FileOperationFailed {
        0: format!("Failed to create cache directory: {}", e),
    })?;

    // Generate blob path
    let blob_path = cache_dir.join(format!("{}.blob", entry_id));

    // Write blob to disk
    fs::write(&blob_path, &data)
        .await
        .map_err(|e| CacheError::FileOperationFailed {
            0: format!("Failed to write blob to disk: {}", e),
        })?;

    // Update or insert cache entry
    upsert_cache_entry(
        db,
        entry_id,
        Some(blob_path.to_string_lossy().to_string()),
        data.len() as u64,
        cloud_version,
        &cloud_checksum,
        true, // is_synced - we just fetched from cloud
        false, // evicted
    )
    .await?;

    tracing::debug!(
        entry_id = %entry_id,
        size_bytes = data.len(),
        "Fetched from cloud and cached"
    );

    // Get the updated entry
    let entry = get_cache_entry(db, entry_id)
        .await?
        .ok_or_else(|| CacheError::EntryNotFound(entry_id))?;

    Ok(CacheRead {
        data,
        entry,
        from_cache: false,
    })
}

/// Retrieves a cache entry by ID without reading the blob.
///
/// This is useful for checking if an entry exists and getting its metadata
/// without incurring the I/O cost of reading the blob.
///
/// # Arguments
///
/// * `db` - SQLite connection pool
/// * `entry_id` - UUID of the cache entry
///
/// # Returns
///
/// Returns `Ok(Some(entry))` if found, `Ok(None)` if not found.
pub async fn get_cache_entry(
    db: &SqlitePool,
    entry_id: Uuid,
) -> Result<Option<LocalCacheEntry>, CacheError> {
    let row = sqlx::query(
        r#"
        SELECT id, blob_path, size_bytes, cloud_version, cloud_checksum,
               is_synced, evicted, last_accessed_at
        FROM local_cache
        WHERE id = ?
        "#,
    )
    .bind(entry_id.to_string())
    .fetch_optional(db)
    .await
    .map_err(|e| CacheError::DatabaseOperationFailed(e.to_string()))?;

    match row {
        Some(r) => match LocalCacheEntry::try_from(r) {
            Ok(entry) => Ok(Some(entry)),
            Err(e) => Err(CacheError::DatabaseOperationFailed(e.to_string())),
        },
        None => Ok(None),
    }
}

/// Reads the blob data from the local filesystem.
///
/// # Arguments
///
/// * `blob_path` - Path to the blob file
///
/// # Returns
///
/// Returns the blob data as bytes.
///
/// # Errors
///
/// Returns `CacheError::FileOperationFailed` if the file cannot be read.
async fn read_blob_from_disk(blob_path: &str) -> Result<Vec<u8>, CacheError> {
    fs::read(blob_path)
        .await
        .map_err(|e| CacheError::FileOperationFailed {
            0: format!("Failed to read blob from {}: {}", blob_path, e),
        })
}

/// Updates the last_accessed_at timestamp for a cache entry.
///
/// This function is called on every cache read to maintain accurate
/// LRU ordering for eviction.
///
/// # Arguments
///
/// * `db` - SQLite connection pool
/// * `entry_id` - UUID of the cache entry
///
/// # Returns
///
/// Returns `Ok(())` on success.
pub async fn update_last_accessed(
    db: &SqlitePool,
    entry_id: Uuid,
) -> Result<(), CacheError> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE local_cache
        SET last_accessed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(now)
    .bind(entry_id.to_string())
    .execute(db)
    .await
    .map_err(|e| CacheError::DatabaseOperationFailed(e.to_string()))?;

    Ok(())
}

/// Gets the total size of all cached blobs in bytes.
///
/// This is used to check if the cache has exceeded the size limit
/// and needs eviction.
///
/// # Arguments
///
/// * `db` - SQLite connection pool
///
/// # Returns
///
/// Returns the total size in bytes.
pub async fn get_cache_size(db: &SqlitePool) -> Result<u64, CacheError> {
    let result: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(size_bytes), 0)
        FROM local_cache
        WHERE blob_path IS NOT NULL
        "#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| CacheError::SizeCalculationFailed(e.to_string()))?;

    Ok(result as u64)
}

/// Gets multiple cache entries by their IDs.
///
/// This is useful for batch operations like preloading or validation.
///
/// # Arguments
///
/// * `db` - SQLite connection pool
/// * `entry_ids` - List of UUIDs to retrieve
///
/// # Returns
///
/// Returns a vector of found entries (order preserved, missing entries omitted).
pub async fn get_cache_entries(
    db: &SqlitePool,
    entry_ids: &[Uuid],
) -> Result<Vec<LocalCacheEntry>, CacheError> {
    if entry_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Build parameterized query with placeholders
    let placeholders: String = entry_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
        r#"
        SELECT id, blob_path, size_bytes, cloud_version, cloud_checksum,
               is_synced, evicted, last_accessed_at
        FROM local_cache
        WHERE id IN ({})
        "#,
        placeholders
    );

    // Create query with bound parameters
    let mut query = sqlx::query_as(&query);
    for id in entry_ids {
        query = query.bind(id.to_string());
    }

    let entries = query
        .fetch_all(db)
        .await
        .map_err(|e| CacheError::DatabaseOperationFailed(e.to_string()))?;

    Ok(entries)
}

/// Checks if a cache entry exists and is available (not evicted).
///
/// # Arguments
///
/// * `db` - SQLite connection pool
/// * `entry_id` - UUID of the cache entry
///
/// # Returns
///
/// Returns `true` if the entry exists and is available.
pub async fn is_cached(db: &SqlitePool, entry_id: Uuid) -> Result<bool, CacheError> {
    let result: Option<(i32,)> = sqlx::query_as(
        r#"
        SELECT 1 FROM local_cache
        WHERE id = ? AND evicted = 0 AND blob_path IS NOT NULL
        "#,
    )
    .bind(entry_id.to_string())
    .fetch_optional(db)
    .await
    .map_err(|e| CacheError::DatabaseOperationFailed(e.to_string()))?;

    Ok(result.is_some())
}

/// Gets the cache directory path, creating it if necessary.
///
/// # Returns
///
/// Returns the path to the cache directory.
async fn get_cache_directory() -> Result<PathBuf, CacheError> {
    // Use standard cache location based on platform
    let cache_dir = match std::env::var("CACHE_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            // Default locations based on OS
            if cfg!(target_os = "linux") {
                PathBuf::from("/var/cache/sync-system")
            } else if cfg!(target_os = "macos") {
                let mut home = std::env::home_dir().unwrap_or_default();
                home.push("Library/Caches/sync-system");
                home
            } else {
                let mut temp = std::env::temp_dir();
                temp.push("sync-system-cache");
                temp
            }
        }
    };

    Ok(cache_dir)
}

/// Creates or updates a cache entry in the database.
///
/// This is used when fetching from cloud to store the entry metadata.
///
/// # Arguments
///
/// * `db` - SQLite connection pool
/// * `id` - Entry UUID
/// * `blob_path` - Path to the local blob file
/// * `size_bytes` - Size of the blob
/// * `cloud_version` - Cloud storage version
/// * `cloud_checksum` - SHA-256 checksum (hex encoded)
/// * `is_synced` - Whether synced to cloud
/// * `evicted` - Whether evicted
async fn upsert_cache_entry(
    db: &SqlitePool,
    id: Uuid,
    blob_path: Option<String>,
    size_bytes: u64,
    cloud_version: i64,
    cloud_checksum: &str,
    is_synced: bool,
    evicted: bool,
) -> Result<(), CacheError> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            blob_path = excluded.blob_path,
            size_bytes = excluded.size_bytes,
            cloud_version = excluded.cloud_version,
            cloud_checksum = excluded.cloud_checksum,
            is_synced = excluded.is_synced,
            evicted = excluded.evicted,
            last_accessed_at = excluded.last_accessed_at
        "#,
    )
    .bind(id.to_string())
    .bind(blob_path.as_deref())
    .bind(size_bytes as i64)
    .bind(cloud_version)
    .bind(cloud_checksum)
    .bind(if is_synced { 1i32 } else { 0i32 })
    .bind(if evicted { 1i32 } else { 0i32 })
    .bind(now)
    .execute(db)
    .await
    .map_err(|e| CacheError::DatabaseOperationFailed(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use tempfile::TempDir;

    /// Creates an in-memory SQLite database for testing.
    async fn create_test_db() -> Result<SqlitePool, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .connect(":memory:")
            .await?;
        crate::schema::initialize_schema(&pool).await?;
        Ok(pool)
    }

    /// Creates a temporary directory for blob storage.
    fn create_temp_cache_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    #[tokio::test]
    async fn test_get_cache_entry() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;
        let entry_id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        // Insert a test entry
        sqlx::query(
            r#"
            INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry_id.to_string())
        .bind("/path/to/blob".to_string())
        .bind(1024i64)
        .bind(1i64)
        .bind("abc123".to_string())
        .bind(1i32)
        .bind(0i32)
        .bind(now)
        .execute(&pool)
        .await?;

        // Retrieve the entry
        let entry = get_cache_entry(&pool, entry_id).await.unwrap();
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.id, entry_id);
        assert_eq!(entry.blob_path, Some("/path/to/blob".to_string()));
        assert_eq!(entry.size_bytes, 1024);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_entry_not_found() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;
        let entry_id = Uuid::new_v4();

        // Try to retrieve non-existent entry
        let entry = get_cache_entry(&pool, entry_id).await.unwrap();
        assert!(entry.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_update_last_accessed() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;
        let entry_id = Uuid::new_v4();
        let original_time = (chrono::Utc::now() - chrono::Duration::seconds(60)).to_rfc3339();

        // Insert a test entry
        sqlx::query(
            r#"
            INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry_id.to_string())
        .bind("/path/to/blob".to_string())
        .bind(1024i64)
        .bind(1i64)
        .bind("abc123".to_string())
        .bind(1i32)
        .bind(0i32)
        .bind(original_time.clone())
        .execute(&pool)
        .await?;

        // Update last_accessed_at
        update_last_accessed(&pool, entry_id).await.unwrap();

        // Verify the timestamp was updated
        let entry = get_cache_entry(&pool, entry_id).await.unwrap().unwrap();
        let updated_time = chrono::DateTime::parse_from_rfc3339(&entry.last_accessed_at.to_rfc3339())
            .unwrap()
            .with_timezone(&chrono::Utc);
        let original = chrono::DateTime::parse_from_rfc3339(&original_time)
            .unwrap()
            .with_timezone(&chrono::Utc);

        assert!(updated_time > original);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_size() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;
        let now = chrono::Utc::now().to_rfc3339();

        // Insert multiple entries
        for i in 0..5 {
            sqlx::query(
                r#"
                INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(Uuid::new_v4().to_string())
            .bind(format!("/path/to/blob{}", i))
            .bind((1024 * (i + 1)) as i64)
            .bind(i as i64)
            .bind(format!("checksum{}", i))
            .bind(1i32)
            .bind(0i32)
            .bind(&now)
            .execute(&pool)
            .await?;
        }

        // Calculate expected size: 1024 + 2048 + 3072 + 4096 + 5120 = 15360
        let size = get_cache_size(&pool).await.unwrap();
        assert_eq!(size, 15360);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_cache_entries_batch() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;
        let now = chrono::Utc::now().to_rfc3339();
        let ids: [Uuid; 3] = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

        // Insert entries
        for (i, &id) in ids.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id.to_string())
            .bind(format!("/path/to/blob{}", i))
            .bind(1024i64)
            .bind(i as i64)
            .bind(format!("checksum{}", i))
            .bind(1i32)
            .bind(0i32)
            .bind(&now)
            .execute(&pool)
            .await?;
        }

        // Retrieve batch
        let entries = get_cache_entries(&pool, &ids).await.unwrap();
        assert_eq!(entries.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_is_cached() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;
        let entry_id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        // Insert a cached entry
        sqlx::query(
            r#"
            INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry_id.to_string())
        .bind("/path/to/blob".to_string())
        .bind(1024i64)
        .bind(1i64)
        .bind("abc123".to_string())
        .bind(1i32)
        .bind(0i32) // not evicted
        .bind(now)
        .execute(&pool)
        .await?;

        // Check if cached
        assert!(is_cached(&pool, entry_id).await.unwrap());

        // Mark as evicted
        sqlx::query("UPDATE local_cache SET evicted = 1 WHERE id = ?")
            .bind(entry_id.to_string())
            .execute(&pool)
            .await?;

        // Should no longer be considered cached
        assert!(!is_cached(&pool, entry_id).await.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_fetch_blob_cache_hit() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;
        let temp_dir = create_temp_cache_dir();
        let entry_id = Uuid::new_v4();
        let blob_path = temp_dir.path().join(format!("{}.blob", entry_id));
        let now = chrono::Utc::now().to_rfc3339();

        // Write test blob to disk
        let test_data = b"Hello, World!";
        fs::write(&blob_path, test_data).await.unwrap();

        // Insert cache entry
        sqlx::query(
            r#"
            INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry_id.to_string())
        .bind(blob_path.to_str())
        .bind(test_data.len() as i64)
        .bind(1i64)
        .bind("abc123".to_string())
        .bind(1i32)
        .bind(0i32)
        .bind(now)
        .execute(&pool)
        .await?;

        // Mock cloud fetcher (should not be called)
        let cloud_fetcher = |_id: Uuid| async move {
            panic!("Cloud fetcher should not be called on cache hit");
        };

        // Get from cache
        let result = get_or_fetch_blob(&pool, entry_id, cloud_fetcher).await.unwrap();
        assert!(result.from_cache);
        assert_eq!(result.data, test_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_fetch_blob_cache_miss() -> Result<(), sqlx::Error> {
        let pool = create_test_db().await?;
        let entry_id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        // Insert cache entry without blob_path (simulating evicted or missing)
        sqlx::query(
            r#"
            INSERT INTO local_cache (id, blob_path, size_bytes, cloud_version, cloud_checksum, is_synced, evicted, last_accessed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry_id.to_string())
        .bind::<Option<&str>>(None) // No blob_path
        .bind(1024i64)
        .bind(1i64)
        .bind("abc123".to_string())
        .bind(1i32)
        .bind(0i32)
        .bind(now)
        .execute(&pool)
        .await?;

        // Mock cloud fetcher
        let test_data = b"Cloud fetched data";
        let cloud_fetcher = |_id: Uuid| async move {
            Ok((test_data.to_vec(), 1, "checksum".to_string()))
        };

        // Fetch from cloud
        let result = get_or_fetch_blob(&pool, entry_id, cloud_fetcher).await.unwrap();
        assert!(!result.from_cache);
        assert_eq!(result.data, test_data);

        Ok(())
    }
}