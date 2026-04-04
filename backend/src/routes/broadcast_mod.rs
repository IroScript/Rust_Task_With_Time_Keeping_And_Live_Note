//! Broadcast system for real-time document synchronization.
//!
//! This module provides document-specific broadcast channels using tokio's
//! broadcast channel for pub/sub messaging to WebSocket subscribers.
//!
//! # Best-Effort Delivery
//!
//! The broadcast system uses best-effort delivery, meaning disconnected
//! clients are skipped without blocking delivery to active clients.
//! This ensures low latency broadcasting as required by Requirement 15.5.

use std::sync::Arc;
use tokio::sync::broadcast;
use crate::routes::AppState;
use shared::SyncMessage;

/// Gets or creates a broadcast channel for a specific document.
///
/// If the document doesn't have a channel yet, a new one is created
/// with a sender capacity of 1000.
///
/// # Arguments
/// * `state` - Application state containing doc_channels map
/// * `doc_id` - Document identifier
///
/// # Returns
/// The broadcast sender for the document's channel
pub fn get_or_create_channel(
    state: &Arc<AppState>,
    doc_id: String,
) -> broadcast::Sender<SyncMessage> {
    state
        .doc_channels
        .entry(doc_id.clone())
        .or_insert_with(|| {
            tracing::debug!(doc_id = %doc_id, "Creating new broadcast channel for document");
            broadcast::channel(1000).0
        })
        .clone()
}

/// Subscribes a receiver to a document's broadcast channel.
///
/// This should be called when a WebSocket client connects to a document.
/// The receiver will receive all broadcast messages for that document.
///
/// # Arguments
/// * `state` - Application state containing doc_channels map
/// * `doc_id` - Document identifier
///
/// # Returns
/// A receiver that will receive broadcast messages for the document
pub fn subscribe(
    state: &Arc<AppState>,
    doc_id: String,
) -> broadcast::Receiver<SyncMessage> {
    let sender = get_or_create_channel(state, doc_id);
    sender.subscribe()
}

/// Broadcasts a message to all subscribers of a document.
///
/// Uses best-effort delivery - disconnected or slow subscribers
/// are skipped without blocking delivery to other active clients.
///
/// # Arguments
/// * `state` - Application state containing doc_channels map
/// * `doc_id` - Document identifier
/// * `message` - Message to broadcast
///
/// # Returns
/// Number of active subscribers the message was sent to
#[allow(dead_code)]
pub fn broadcast(state: &Arc<AppState>, doc_id: String, message: SyncMessage) -> usize {
    if let Some(sender) = state.doc_channels.get(&doc_id) {
        // Best-effort delivery: ignore send errors (disconnected clients)
        // This ensures low latency as required by Requirement 15.5
        let result = sender.send(message);
        
        match result {
            Ok(count) => {
                tracing::debug!(doc_id = %doc_id, subscribers = count, "Broadcast sent to subscribers");
                count
            }
            Err(_) => {
                tracing::debug!(doc_id = %doc_id, "Broadcast failed - no active subscribers");
                0
            }
        }
    } else {
        tracing::debug!(doc_id = %doc_id, "No broadcast channel exists for document");
        0
    }
}

// Comment out unused function
// /// Broadcasts a delta message to all subscribers of a document.
// ///
// /// This is a convenience function that constructs the broadcast message
// /// and sends it to all subscribers.
// ///
// /// # Arguments
// /// * `state` - Application state containing doc_channels map
// /// * `doc_id` - Document identifier
// /// * `version` - CRDT version number
// /// * `update` - yrs-encoded CRDT update bytes
// /// * `author_id` - User who authored this update
// ///
// /// # Returns
// /// Number of active subscribers the message was sent to
// pub fn broadcast_delta(
#[allow(dead_code)]
fn broadcast_delta(
    state: &Arc<AppState>,
    doc_id: String,
    version: i64,
    update: Vec<u8>,
    author_id: String,
) -> usize {
    let doc_id_clone = doc_id.clone();
    let message = SyncMessage::Delta {
        doc_id,
        version,
        update,
        author_id,
    };
    broadcast(state, doc_id_clone, message)
}

// Comment out unused function
// /// Removes a broadcast channel for a document.
// ///
// /// This should be called when the last subscriber disconnects
// /// to clean up resources. The channel will be removed from the map,
// /// and any remaining receivers will receive a close signal.
// ///
// /// # Arguments
// /// * `state` - Application state containing doc_channels map
// /// * `doc_id` - Document identifier
// ///
// /// # Returns
// /// Whether a channel was removed
// pub fn remove_channel(state: &Arc<AppState>, doc_id: Uuid) -> bool {
#[allow(dead_code)]
fn remove_channel(state: &Arc<AppState>, doc_id: String) -> bool {
    let removed = state.doc_channels.remove(&doc_id).is_some();
    if removed {
        tracing::debug!(doc_id = %doc_id, "Removed broadcast channel for document");
    }
    removed
}

// Comment out unused function
// /// Gets the current number of active subscribers for a document.
// ///
// /// Note: This returns the sender's subscriber count, which may include
// /// subscribers that have disconnected but haven't been cleaned up yet.
// ///
// /// # Arguments
// /// * `state` - Application state containing doc_channels map
// /// * `doc_id` - Document identifier
// ///
// /// # Returns
// /// Number of active subscribers, or 0 if no channel exists
// pub fn subscriber_count(state: &Arc<AppState>, doc_id: Uuid) -> usize {
#[allow(dead_code)]
fn subscriber_count(state: &Arc<AppState>, doc_id: String) -> usize {
    state
        .doc_channels
        .get(&doc_id)
        .map(|sender| sender.receiver_count())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use dashmap::DashMap;
    use tokio::sync::broadcast;

    fn create_test_state() -> Arc<AppState> {
        Arc::new(AppState {
            db_pool: sqlx::PgPool::connect_lazy("postgres://test:test@localhost/test").unwrap(),
            crdt_docs: Arc::new(DashMap::new()),
            connections: Arc::new(DashMap::new()),
            doc_channels: Arc::new(DashMap::new()),
        })
    }

    #[tokio::test]
    async fn test_channel_creation() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Channel should not exist yet
        assert_eq!(subscriber_count(&state, doc_id), 0);

        // Create channel
        let sender = get_or_create_channel(&state, doc_id);
        assert_eq!(sender.receiver_count(), 0);

        // Subscribe a receiver
        let _receiver = sender.subscribe(); // Use sender.subscribe() directly
        assert_eq!(sender.receiver_count(), 1);
    }

    #[tokio::test]
    async fn test_broadcast_to_single_subscriber() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Create channel and subscribe
        let _sender = get_or_create_channel(&state, doc_id);
        let mut receiver = subscribe(&state, doc_id);

        // Broadcast a message
        let message = SyncMessage::Ack { sequence: 42 };
        let count = broadcast(&state, doc_id, message.clone());
        assert_eq!(count, 1);

        // Receiver should get the message
        let received = receiver.recv().await.unwrap();
        match received {
            SyncMessage::Ack { sequence } => assert_eq!(sequence, 42),
            _ => panic!("Wrong message type received"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_to_multiple_subscribers() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Create channel and subscribe multiple receivers
        let _sender = get_or_create_channel(&state, doc_id);
        let mut receiver1 = subscribe(&state, doc_id);
        let mut receiver2 = subscribe(&state, doc_id);
        let mut receiver3 = subscribe(&state, doc_id);

        // Broadcast a message
        let message = SyncMessage::Ack { sequence: 100 };
        let count = broadcast(&state, doc_id, message.clone());
        assert_eq!(count, 3);

        // All receivers should get the message
        assert_eq!(receiver1.recv().await.unwrap(), message);
        assert_eq!(receiver2.recv().await.unwrap(), message);
        assert_eq!(receiver3.recv().await.unwrap(), message);
    }

    #[tokio::test]
    async fn test_broadcast_no_subscribers() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Create channel but don't subscribe
        let _sender = get_or_create_channel(&state, doc_id);

        // Broadcast should return 0 (no active subscribers)
        let message = SyncMessage::Ack { sequence: 1 };
        let count = broadcast(&state, doc_id, message);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_broadcast_no_channel() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Broadcast without creating a channel first
        let message = SyncMessage::Ack { sequence: 1 };
        let count = broadcast(&state, doc_id, message);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_remove_channel() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Create channel
        get_or_create_channel(&state, doc_id);
        assert!(state.doc_channels.get(&doc_id).is_some());

        // Remove channel
        let removed = remove_channel(&state, doc_id);
        assert!(removed);
        assert!(state.doc_channels.get(&doc_id).is_none());

        // Broadcasting to removed channel should return 0
        let message = SyncMessage::Ack { sequence: 1 };
        let count = broadcast(&state, doc_id, message);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_broadcast_delta() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        // Create channel and subscribe
        let _sender = get_or_create_channel(&state, doc_id);
        let mut receiver = subscribe(&state, doc_id);

        // Broadcast delta
        let update = vec![1, 2, 3, 4];
        let count = broadcast_delta(&state, doc_id, 5, update.clone(), author_id);
        assert_eq!(count, 1);

        // Verify received message
        let received = receiver.recv().await.unwrap();
        match received {
            SyncMessage::Delta {
                doc_id: received_doc_id,
                version,
                update: received_update,
                author_id: received_author_id,
            } => {
                assert_eq!(received_doc_id, doc_id);
                assert_eq!(version, 5);
                assert_eq!(received_update, update);
                assert_eq!(received_author_id, author_id);
            }
            SyncMessage::Presence(_) | SyncMessage::Ack { .. } => {
                // These variants shouldn't occur in this test
                panic!("Unexpected message variant received");
            }
        }
    }
}