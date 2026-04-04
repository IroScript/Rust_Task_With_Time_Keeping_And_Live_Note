//! S3 blob storage operations with checksums and deduplication.
//!
//! This module provides:
//! - Upload/download functions for blob storage
//! - SHA-256 checksum computation and verification
//! - Content-addressable storage for deduplication

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::{debug, info, instrument};

/// Custom error type for S3 operations.
#[derive(Debug, Error)]
pub enum S3Error {
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("S3 operation failed: {0}")]
    S3Operation(String),

    #[error("blob not found: {0}")]
    NotFound(String),

    #[error("invalid blob size: {0}")]
    InvalidSize(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for S3Error {
    fn from(e: std::io::Error) -> Self {
        S3Error::Io(e.to_string())
    }
}

/// Result type for S3 operations.
pub type S3Result<T> = Result<T, S3Error>;

/// Configuration for S3 blob storage.
#[derive(Debug, Clone)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    /// S3 prefix for blob keys (e.g., "blobs/" or "documents/")
    pub prefix: String,
    /// Minimum blob size in bytes to store in S3 (smaller blobs stored inline)
    pub min_blob_size: u64,
    /// Enable content-addressable deduplication
    pub enable_deduplication: bool,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: "sync-system-blobs".to_string(),
            prefix: "blobs/".to_string(),
            min_blob_size: 1024, // 1 KB threshold from requirement 6.2
            enable_deduplication: true,
        }
    }
}

/// A stored blob with its metadata.
#[derive(Debug, Clone)]
pub struct StoredBlob {
    /// Content-addressable key (SHA-256 hash of content)
    pub key: String,
    /// Original size in bytes
    pub size: u64,
    /// SHA-256 checksum of the content
    pub checksum: [u8; 32],
    /// Whether the blob was deduplicated
    pub was_deduplicated: bool,
    /// ETag returned by S3 (for verification)
    pub etag: Option<String>,
}

/// Content hasher utility for computing SHA-256 checksums.
#[derive(Debug, Default)]
pub struct ContentHasher {
    hasher: Sha256,
}

impl ContentHasher {
    #[inline]
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    #[inline]
    pub fn finalize(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }

    #[inline]
    pub fn compute(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    #[inline]
    pub fn to_hex(checksum: &[u8; 32]) -> String {
        hex::encode(checksum)
    }

    #[inline]
    pub fn from_hex(hex: &str) -> Result<[u8; 32], S3Error> {
        let bytes: [u8; 32] = hex::decode(hex)
            .map_err(|e| S3Error::S3Operation(format!("Invalid hex: {}", e)))?
            .try_into()
            .map_err(|_| S3Error::S3Operation("Invalid hex length".to_string()))?;
        Ok(bytes)
    }
}

/// Generate content-addressable key from checksum.
#[inline]
pub fn content_addressable_key(prefix: &str, checksum: &[u8; 32]) -> String {
    let hex = hex::encode(checksum);
    format!("{}{}/{}", prefix, &hex[..2], &hex)
}

/// S3 blob storage operations.
#[derive(Debug, Clone)]
pub struct S3BlobStorage {
    client: S3Client,
    config: S3Config,
}

impl S3BlobStorage {
    #[inline]
    pub fn new(client: S3Client, config: S3Config) -> Self {
        Self { client, config }
    }

    #[inline]
    pub fn blob_key(&self, checksum: &[u8; 32]) -> String {
        content_addressable_key(&self.config.prefix, checksum)
    }

    /// Upload a blob to S3 with deduplication.
    #[instrument(skip(data), fields(key))]
    pub async fn upload_blob(
        &self,
        data: &[u8],
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> S3Result<StoredBlob> {
        let size = data.len() as u64;
        let checksum = ContentHasher::compute(data);
        let checksum_hex = ContentHasher::to_hex(&checksum);
        let key = self.blob_key(&checksum);

        // Check if blob already exists (deduplication)
        if self.config.enable_deduplication {
            if self.blob_exists(&key).await? {
                debug!(key, "blob already exists, skipping upload (deduplication)");
                return Ok(StoredBlob {
                    key,
                    size,
                    checksum,
                    was_deduplicated: true,
                    etag: None,
                });
            }
        }

        // Upload to S3
        let body = ByteStream::from(data.to_vec());

        let mut put_request = self
            .client
            .put_object()
            .bucket(&self.config.bucket)
            .key(&key)
            .body(body)
            .content_type("application/octet-stream");

        // Add checksum as custom metadata for verification
        let mut custom_metadata = std::collections::HashMap::new();
        custom_metadata.insert("checksum".to_string(), checksum_hex.clone());
        custom_metadata.insert("size".to_string(), size.to_string());

        if let Some(meta) = metadata {
            for (k, v) in meta {
                custom_metadata.insert(k, v);
            }
        }

        put_request = put_request.set_metadata(Some(custom_metadata));

        let response = put_request.send().await.map_err(|e| S3Error::S3Operation(e.to_string()))?;

        let etag = response.e_tag.map(|e| e.to_string());

        info!(key, size, checksum = checksum_hex, "blob uploaded to S3");

        Ok(StoredBlob {
            key,
            size,
            checksum,
            was_deduplicated: false,
            etag,
        })
    }

    /// Download a blob from S3.
    #[instrument(skip(expected_checksum), fields(key))]
    pub async fn download_blob(
        &self,
        key: &str,
        expected_checksum: Option<&[u8; 32]>,
    ) -> S3Result<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3Error::S3Operation(e.to_string()))?;

        let data = response.body.collect().await.map_err(|e| S3Error::S3Operation(e.to_string()))?.to_vec();

        // Verify checksum if provided
        if let Some(expected) = expected_checksum {
            let actual = ContentHasher::compute(&data);
            if actual != *expected {
                return Err(S3Error::ChecksumMismatch {
                    expected: ContentHasher::to_hex(expected),
                    actual: ContentHasher::to_hex(&actual),
                });
            }
        }

        debug!(key, size = data.len(), "blob downloaded from S3");

        Ok(data)
    }

    /// Download a blob and verify its checksum against stored metadata.
    #[instrument(skip_all, fields(key))]
    pub async fn download_blob_with_verification(&self, key: &str) -> S3Result<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3Error::S3Operation(e.to_string()))?;

        let metadata = response.metadata().expect("metadata should be present");
        let expected_checksum_hex = metadata
            .get("checksum")
            .ok_or_else(|| S3Error::NotFound(key.to_string()))?;

        let expected_checksum = ContentHasher::from_hex(expected_checksum_hex)?;

        self.download_blob(key, Some(&expected_checksum)).await
    }

    /// Check if a blob exists in S3.
    #[inline]
    pub async fn blob_exists(&self, key: &str) -> S3Result<bool> {
        let result = self
            .client
            .head_object()
            .bucket(&self.config.bucket)
            .key(key)
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("NoSuchKey") || err_str.contains("NotFound") {
                    Ok(false)
                } else {
                    Err(S3Error::S3Operation(err_str))
                }
            }
        }
    }

    /// Delete a blob from S3.
    #[instrument(skip_all, fields(key))]
    pub async fn delete_blob(&self, key: &str) -> S3Result<()> {
        self.client
            .delete_object()
            .bucket(&self.config.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3Error::S3Operation(e.to_string()))?;

        debug!(key, "blob deleted from S3");
        Ok(())
    }

    /// Get blob metadata from S3.
    #[instrument(skip_all, fields(key))]
    pub async fn get_blob_metadata(&self, key: &str) -> S3Result<BlobMetadata> {
        let response = self
            .client
            .head_object()
            .bucket(&self.config.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3Error::S3Operation(e.to_string()))?;

        let metadata = response.metadata().expect("metadata should be present");
        let checksum_hex = metadata
            .get("checksum")
            .ok_or_else(|| S3Error::NotFound(key.to_string()))?;

        let size: u64 = metadata
            .get("size")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok(BlobMetadata {
            key: key.to_string(),
            size,
            checksum: ContentHasher::from_hex(checksum_hex)?,
            last_modified: response.last_modified().map(|dt| {
                chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos() as u32)
                    .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH)
            }),
        })
    }

    /// Verify blob integrity by checking checksum.
    #[instrument(skip_all, fields(key))]
    pub async fn verify_blob_integrity(&self, key: &str) -> S3Result<bool> {
        let metadata = self.get_blob_metadata(key).await?;
        let data = self.download_blob(key, Some(&metadata.checksum)).await?;

        let actual_checksum = ContentHasher::compute(&data);
        Ok(actual_checksum == metadata.checksum)
    }

    /// List all blobs with a given prefix.
    #[instrument(skip_all)]
    pub async fn list_blobs(&self, prefix: Option<&str>) -> S3Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut continuation_token = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.config.bucket);

            if let Some(p) = prefix {
                request = request.prefix(p);
            }

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| S3Error::S3Operation(e.to_string()))?;

            let contents = response.contents();
            for obj in contents {
                if let Some(key) = &obj.key {
                    keys.push(key.clone());
                }
            }

            if response.is_truncated().expect("is_truncated should be present") {
                continuation_token = response.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(keys)
    }
}

/// Metadata for a stored blob.
#[derive(Debug, Clone)]
pub struct BlobMetadata {
    pub key: String,
    pub size: u64,
    pub checksum: [u8; 32],
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

/// Upload manager for handling concurrent uploads with deduplication.
#[derive(Debug)]
pub struct UploadManager {
    storage: S3BlobStorage,
    in_progress: std::sync::Arc<dashmap::DashMap<String, tokio::sync::Mutex<()>>>,
}

impl UploadManager {
    #[inline]
    pub fn new(storage: S3BlobStorage) -> Self {
        Self {
            storage,
            in_progress: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Upload a blob with deduplication and concurrency control.
    pub async fn upload(
        &self,
        data: &[u8],
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> S3Result<StoredBlob> {
        let checksum = ContentHasher::compute(data);
        let key = self.storage.blob_key(&checksum);

        let guard = self.in_progress.entry(key.clone()).or_insert_with(|| {
            tokio::sync::Mutex::new(())
        });

        let _guard = guard.value().lock().await;

        if self.storage.config.enable_deduplication && self.storage.blob_exists(&key).await? {
            return Ok(StoredBlob {
                key,
                size: data.len() as u64,
                checksum,
                was_deduplicated: true,
                etag: None,
            });
        }

        let result = self.storage.upload_blob(data, metadata).await;
        self.in_progress.remove(&key);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hasher() {
        let data = b"hello world";
        let checksum = ContentHasher::compute(data);
        assert_eq!(checksum.len(), 32);

        let hex = ContentHasher::to_hex(&checksum);
        assert_eq!(hex.len(), 64);

        let parsed = ContentHasher::from_hex(&hex).unwrap();
        assert_eq!(parsed, checksum);
    }

    #[test]
    fn test_content_addressable_key() {
        let checksum = ContentHasher::compute(b"test data");
        let key = content_addressable_key("blobs/", &checksum);

        assert!(key.starts_with("blobs/"));
        assert!(key.contains("/"));
        assert_eq!(key.len(), 6 + 2 + 1 + 64);
    }

    #[test]
    fn test_checksum_mismatch_error() {
        let error = S3Error::ChecksumMismatch {
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };

        assert!(error.to_string().contains("checksum mismatch"));
    }
}