//! Local cache module for the Pure Rust Backend Sync System.
//!
//! This module provides SQLite-based local caching for document blobs
//! with support for LRU eviction and cloud synchronization tracking.

pub mod schema;
pub mod read;