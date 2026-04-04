//! Error types for the Pure Rust Backend Sync System
//!
//! This module defines structured error types using thiserror for
//! proper error handling across all system components.

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during sync operations.
#[derive(Error, Debug)]
pub enum SyncError {
    /// WebSocket connection was closed unexpectedly
    #[error("WebSocket connection closed")]
    ConnectionClosed,

    /// Received an invalid message format
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Message type is not supported
    #[error("Unsupported message type")]
    UnsupportedMessageType,

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authorization check failed
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    /// Document not found
    #[error("Document not found: {0}")]
    DocumentNotFound(Uuid),

    /// Version conflict detected
    #[error("Version conflict for document {doc_id}: expected {expected}, got {actual}")]
    VersionConflict {
        /// Document ID
        doc_id: Uuid,
        /// Expected version
        expected: i64,
        /// Actual version received
        actual: i64,
    },

    /// Broadcast channel closed
    #[error("Broadcast channel closed for document {0}")]
    BroadcastChannelClosed(Uuid),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// CRDT operation error
    #[error("CRDT error: {0}")]
    CRDTError(String),

    /// Serialization failed
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    /// Failed to send message
    #[error("Failed to send message: {0}")]
    SendFailed(String),
}

/// Errors that can occur during Vault operations.
#[derive(Error, Debug)]
pub enum VaultError {
    /// Failed to connect to Vault server
    #[error("Failed to connect to Vault: {0}")]
    ConnectionFailed(String),

    /// Vault request failed
    #[error("Vault request failed: {0}")]
    RequestFailed(String),

    /// Invalid credentials received from Vault
    #[error("Invalid credentials received from Vault")]
    InvalidCredentials,

    /// Vault authentication failed
    #[error("Vault authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Role not found in Vault configuration
    #[error("Database role not found: {0}")]
    RoleNotFound(String),

    /// Credential TTL exceeds maximum allowed
    #[error("Credential TTL {ttl} exceeds maximum of {max_ttl}")]
    TtlExceeded {
        /// Received TTL
        ttl: u64,
        /// Maximum allowed TTL
        max_ttl: u64,
    },

    /// No cached credentials available
    #[error("No cached credentials available")]
    NoCachedCredentials,
}

/// Errors that can occur during CRDT operations.
#[derive(Error, Debug)]
pub enum CRDTError {
    /// Invalid CRDT update format
    #[error("Invalid CRDT update: {0}")]
    InvalidUpdate(String),

    /// CRDT merge operation failed
    #[error("CRDT merge failed: {0}")]
    MergeFailed(String),

    /// Document state encoding failed
    #[error("Failed to encode document state: {0}")]
    EncodingFailed(String),

    /// Document state decoding failed
    #[error("Failed to decode document state: {0}")]
    DecodingFailed(String),

    /// Document not found in memory
    #[error("CRDT document not found: {0}")]
    DocumentNotFound(Uuid),

    /// Transaction failed
    #[error("CRDT transaction failed: {0}")]
    TransactionFailed(String),
}

/// Errors that can occur during storage operations.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Failed to establish database connection
    #[error("Failed to connect to PostgreSQL at {host}:{port}: {message}")]
    ConnectionFailed {
        /// Database host
        host: String,
        /// Database port
        port: u16,
        /// Error message
        message: String,
    },

    /// Invalid configuration setting
    #[error("Invalid configuration for {setting}: {message}")]
    ConfigurationError {
        /// Configuration setting name
        setting: String,
        /// Error message
        message: String,
    },

    /// Database query execution failed
    #[error("Database query failed: {0}")]
    QueryFailed(String),

    /// Transaction failed
    #[error("Transaction failed during {operation}: {message}")]
    TransactionFailed {
        /// Operation that failed
        operation: String,
        /// Error message
        message: String,
    },

    /// S3 operation failed
    #[error("S3 operation {operation} failed: {message}")]
    S3Operation {
        /// S3 operation that failed
        operation: String,
        /// Error message
        message: String,
    },

    /// Blob not found
    #[error("Blob not found: {0}")]
    BlobNotFound(String),

    /// Version conflict detected during optimistic locking
    #[error("Version conflict for document {doc_id}: expected {expected_version}, got {actual_version}")]
    VersionConflict {
        /// Document ID
        doc_id: Uuid,
        /// Expected version
        expected_version: i64,
        /// Actual version in database
        actual_version: i64,
    },

    /// Blob exceeds maximum allowed size
    #[error("Blob size {size} exceeds maximum allowed size {max_size}")]
    BlobTooLarge {
        /// Actual blob size
        size: i64,
        /// Maximum allowed size
        max_size: i64,
    },
}

/// Errors that can occur during cache operations.
#[derive(Error, Debug)]
pub enum CacheError {
    /// Cache size calculation failed
    #[error("Failed to calculate cache size: {0}")]
    SizeCalculationFailed(String),

    /// Cloud verification failed
    #[error("Cloud verification failed for entry {id}: {reason}")]
    CloudVerificationFailed {
        /// Cache entry ID
        id: Uuid,
        /// Reason for failure
        reason: String,
    },

    /// File operation failed
    #[error("File operation failed: {0}")]
    FileOperationFailed(String),

    /// Database operation failed
    #[error("Database operation failed: {0}")]
    DatabaseOperationFailed(String),

    /// Cache is full and cannot evict enough space
    #[error("Cache full: cannot free enough space (needed {needed}, available {available})")]
    CacheFull {
        /// Bytes needed to free
        needed: u64,
        /// Bytes currently available
        available: u64,
    },

    /// Entry not found in cache
    #[error("Cache entry not found: {0}")]
    EntryNotFound(Uuid),

    /// Eviction not allowed (entry is not synced)
    #[error("Cannot evict unsynced entry: {0}")]
    CannotEvictUnsynced(Uuid),
}