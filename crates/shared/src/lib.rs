//! Shared types and utilities for the Pure Rust Backend Sync System
//!
//! This crate provides core types and error definitions used across
//! all system components: gateway, sync-engine, storage, local-cache, and auth.
//!
//! # Modules
//! * `types` - Core data structures (Document, SyncMessage, PresenceInfo, etc.)
//! * `error` - Error types using thiserror (SyncError, VaultError, CRDTError, CacheError)
#![forbid(unsafe_code)]

pub mod types;
pub mod error;

// Re-export commonly used types for convenience
pub use types::{
    AppState, CacheConfig, CacheEntry, DatabaseCredentials, Document, EvictionPolicy,
    PresenceInfo, SyncMessage,
};
pub use error::{CacheError, CRDTError, StorageError, SyncError, VaultError};