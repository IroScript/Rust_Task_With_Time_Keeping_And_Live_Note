//! Core shared types for the Pure Rust Backend Sync System
//!
//! This module defines the fundamental data structures used across all crates
//! including documents, sync messages, presence information, and cache entries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core document metadata representing a synchronized document.
///
/// # Fields
/// * `id` - Unique identifier for the document
/// * `workspace_id` - Workspace this document belongs to
/// * `owner_id` - User who owns the document
/// * `title` - Document title
/// * `content_hash` - SHA-256 hash of the document content
/// * `blob_ref` - S3 key reference to the document blob
/// * `crdt_version` - Current CRDT version number
/// * `size_bytes` - Size of the document in bytes
/// * `mime_type` - MIME type of the document content
/// * `created_at` - Timestamp when document was created
/// * `updated_at` - Timestamp when document was last updated
/// * `deleted_at` - Timestamp when document was soft-deleted (None if active)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub content_hash: Vec<u8>,
    pub blob_ref: String,
    pub crdt_version: i64,
    pub size_bytes: i64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// CRDT sync message variants for real-time synchronization.
///
/// This enum represents the different types of messages exchanged
/// between clients and the sync engine over WebSocket connections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncMessage {
    /// An incremental CRDT update (delta)
    Delta {
        /// Document being updated
        doc_id: String,
        /// Current CRDT version
        version: i64,
        /// yrs-encoded CRDT update bytes
        update: Vec<u8>,
        /// Author of this update
        author_id: String,
    },
    /// Real-time presence update
    Presence(PresenceInfo),
    /// Acknowledgment of a processed message
    Ack {
        /// Sequence number being acknowledged
        sequence: u64,
    },
}

/// Real-time presence information for collaborative editing.
///
/// Tracks user activity indicators like cursor position and typing status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresenceInfo {
    /// User identifier
    pub user_id: String,
    /// User's display name
    pub user_name: String,
    /// Document being viewed/edited
    pub doc_id: String,
    /// Current cursor position in the document
    pub cursor_position: Option<usize>,
    /// Whether the user is currently typing
    pub is_typing: bool,
    /// Last activity timestamp
    pub last_active: DateTime<Utc>,
}

/// Local cache entry for cached document blobs.
///
/// Represents a document blob stored in the local SQLite cache
/// with metadata for eviction and sync tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Unique identifier for this cache entry
    pub id: Uuid,
    /// Path to the local blob file (None if evicted)
    pub blob_path: Option<String>,
    /// Size of the blob in bytes
    pub size_bytes: u64,
    /// Cloud storage version for this entry
    pub cloud_version: i64,
    /// SHA-256 checksum of the cloud blob
    pub cloud_checksum: Vec<u8>,
    /// Whether this entry has been synced to cloud storage
    pub is_synced: bool,
    /// Whether this entry has been evicted
    pub evicted: bool,
    /// Last access timestamp for LRU ordering
    pub last_accessed_at: DateTime<Utc>,
}

/// Configuration for the local cache eviction policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum local cache size in bytes (default: 1 GB)
    pub max_local_bytes: u64,
    /// Number of entries to process per eviction batch
    pub eviction_batch_size: usize,
    /// Eviction policy to use
    pub eviction_policy: EvictionPolicy,
}

/// Available eviction policies for cache management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used - evicts oldest accessed entries first
    LRU,
    /// Least Frequently Used - evicts least accessed entries first
    LFU,
    /// First In First Out - evicts oldest entries first
    FIFO,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_local_bytes: 1024 * 1024 * 1024, // 1 GB
            eviction_batch_size: 100,
            eviction_policy: EvictionPolicy::LRU,
        }
    }
}

/// Credentials obtained from HashiCorp Vault for database connections.
///
/// These are ephemeral credentials with a limited TTL that must be
/// refreshed before expiration.
#[derive(Debug, Clone)]
pub struct DatabaseCredentials {
    /// Database username
    pub username: String,
    /// Database password
    pub password: String,
    /// Time-to-live in seconds (max 3600)
    pub ttl: u64,
    /// Timestamp when credentials were issued
    pub issued_at: DateTime<Utc>,
}

/// Application state shared across the gateway.
///
/// Contains all shared resources needed by request handlers
/// and WebSocket handlers.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db_pool: sqlx::PgPool,
    /// S3 client for blob storage (commented out for now)
    // pub s3_client: aws_sdk_s3::Client,
    /// HashiCorp Vault client (arc-wrapped for sharing)
    // pub vault_client: std::sync::Arc<vaultrs::client::VaultClient>,
    /// Document-specific broadcast channels for WebSocket subscribers
    pub doc_channels: std::sync::Arc<dashmap::DashMap<Uuid, tokio::sync::broadcast::Sender<SyncMessage>>>,
    /// Presence maps per document (doc_id -> Vec<PresenceInfo>)
    pub presence: std::sync::Arc<dashmap::DashMap<Uuid, Vec<PresenceInfo>>>,
    /// In-memory CRDT documents for active documents
    pub crdt_docs: std::sync::Arc<dashmap::DashMap<Uuid, std::sync::Arc<tokio::sync::RwLock<yrs::Doc>>>>,
}