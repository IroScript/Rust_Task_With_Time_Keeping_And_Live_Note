//! Property-based tests for broadcast system best-effort delivery.
//!
//! These tests verify Property 23: Broadcast Best-Effort Delivery
//! Validates: Requirement 15.5
//!
//! The tests use proptest to generate various scenarios and verify that
//! disconnected clients are skipped without blocking delivery to active clients.

use proptest::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::routes::AppState;
use shared::SyncMessage;

/// Property 23: Broadcast Best-Effort Delivery
///
/// *For any* delta broadcast, disconnected clients SHALL be skipped
/// without blocking delivery to active clients.
///
/// This property is verified by:
/// 1. Creating a broadcast channel with multiple subscribers
/// 2. Dropping some subscribers (simulating disconnection)
/// 3. Broadcasting a message
/// 4. Verifying that active subscribers receive the message
/// 5. Verifying that the broadcast doesn't block on disconnected clients

/// Generator for valid document IDs
fn doc_id_strategy() -> impl Strategy<Value = Uuid> {
    proptest::just(Uuid::new_v4())
}

/// Generator for valid author IDs
fn author_id_strategy() -> impl Strategy<Value = Uuid> {
    proptest::just(Uuid::new_v4())
}

/// Generator for CRDT update bytes (non-empty)
fn update_strategy() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(proptest::num::u8::ANY, 1..100)
}

/// Generator for version numbers (positive)
fn version_strategy() -> impl Strategy<Value = i64> {
    proptest::num::pos_i64()
}

/// Generator for sequence numbers (positive)
fn sequence_strategy() -> impl Strategy<Value = u64> {
    proptest::num::pos_u64()
}

/// Creates a test state with the given number of document channels
fn create_test_state_with_channels(num_channels: usize) -> Arc<AppState> {
    Arc::new(AppState {
        db_pool: sqlx::PgPool::connect_lazy("postgres://test:test@localhost/test").unwrap(),
        crdt_docs: Arc::new(dashmap::DashMap::new()),
        connections: Arc::new(dashmap::DashMap::new()),
        doc_channels: Arc::new(dashmap::DashMap::new()),
    })
}

/// Test that broadcast returns correct subscriber count
///
/// For any broadcast, the return value should equal the number of
/// active subscribers at the time of broadcast.
proptest! {
    #[test]
    fn test_broadcast_returns_subscriber_count(
        num_subscribers in 1..10usize,
    ) {
        let state = create_test_state_with_channels(1);
        let doc_id = Uuid::new_v4();

        // Create channel and subscribe receivers
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        let mut receivers: Vec<broadcast::Receiver<SyncMessage>> = (0..num_subscribers)
            .map(|_| sender.subscribe())
            .collect();

        // Broadcast a message
        let message = SyncMessage::Ack { sequence: 42 };
        let count = sender.send(message.clone());

        // Count should match number of receivers
        prop_assert_eq!(count, num_subscribers);

        // All active receivers should get the message
        for receiver in &mut receivers {
            let received = receiver.recv_timeout(Duration::from_millis(100));
            prop_assert!(received.is_ok());
            if let Ok(msg) = received {
                prop_assert_eq!(msg, message);
            }
        }
    }
}

/// Test that disconnected clients don't block broadcast
///
/// When subscribers are dropped before broadcast, the broadcast
/// should complete without blocking and return the correct count.
proptest! {
    #[test]
    fn test_disconnected_clients_skipped(
        total_subscribers in 5..20usize,
        disconnect_count in 0..5usize,
    ) {
        let state = create_test_state_with_channels(1);
        let doc_id = Uuid::new_v4();

        // Create channel and subscribe all receivers
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        let mut receivers: Vec<broadcast::Receiver<SyncMessage>> = (0..total_subscribers)
            .map(|_| sender.subscribe())
            .collect();

        // Drop some receivers (simulating disconnection)
        let active_count = total_subscribers - disconnect_count;
        receivers.truncate(active_count);

        // Broadcast should complete immediately
        let message = SyncMessage::Ack { sequence: 1 };
        let start = std::time::Instant::now();
        let count = sender.send(message.clone());
        let elapsed = start.elapsed();

        // Broadcast should complete quickly (within 100ms)
        prop_assert!(elapsed < Duration::from_millis(100),
            "Broadcast took too long: {:?}", elapsed);

        // Count should match active subscribers
        prop_assert_eq!(count, active_count);

        // All active receivers should get the message
        for receiver in &mut receivers {
            let received = receiver.recv_timeout(Duration::from_millis(50));
            prop_assert!(received.is_ok());
        }
    }
}

/// Test that broadcast works with zero active subscribers
///
/// Broadcasting to a channel with no active subscribers should
/// return 0 and complete immediately.
proptest! {
    #[test]
    fn test_broadcast_no_subscribers(
        _doc_id in doc_id_strategy(),
    ) {
        let state = create_test_state_with_channels(1);
        let doc_id = Uuid::new_v4();

        // Create channel but don't subscribe
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        // Broadcast should return 0
        let message = SyncMessage::Ack { sequence: 1 };
        let count = sender.send(message);

        prop_assert_eq!(count, 0);
    }
}

/// Test that delta broadcasts work correctly
///
/// Delta messages should be broadcast to all active subscribers
/// with correct content preserved.
proptest! {
    #[test]
    fn test_delta_broadcast_content(
        doc_id in doc_id_strategy(),
        version in version_strategy(),
        update in update_strategy(),
        author_id in author_id_strategy(),
        num_receivers in 1..5usize,
    ) {
        let state = create_test_state_with_channels(1);

        // Create channel and subscribe
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        let mut receivers: Vec<broadcast::Receiver<SyncMessage>> = (0..num_receivers)
            .map(|_| sender.subscribe())
            .collect();

        // Create and broadcast delta
        let message = SyncMessage::Delta {
            doc_id,
            version,
            update: update.clone(),
            author_id,
        };
        let count = sender.send(message.clone());

        prop_assert_eq!(count, num_receivers);

        // Verify content for each receiver
        for receiver in &mut receivers {
            let received = receiver.recv_timeout(Duration::from_millis(100));
            prop_assert!(received.is_ok());

            if let Ok(SyncMessage::Delta {
                doc_id: received_doc_id,
                version: received_version,
                update: received_update,
                author_id: received_author_id,
            }) = received
            {
                prop_assert_eq!(received_doc_id, doc_id);
                prop_assert_eq!(received_version, version);
                prop_assert_eq!(received_update, update);
                prop_assert_eq!(received_author_id, author_id);
            } else {
                prop_assert!(false, "Expected Delta message");
            }
        }
    }
}

/// Test that presence broadcasts work correctly
///
/// Presence messages should be broadcast to all active subscribers.
proptest! {
    #[test]
    fn test_presence_broadcast(
        doc_id in doc_id_strategy(),
        user_id in author_id_strategy(),
        num_receivers in 1..5usize,
        is_typing in proptest::bool::ANY,
        cursor_position in proptest::option::of(0..1000usize),
    ) {
        let state = create_test_state_with_channels(1);

        // Create channel and subscribe
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        let mut receivers: Vec<broadcast::Receiver<SyncMessage>> = (0..num_receivers)
            .map(|_| sender.subscribe())
            .collect();

        // Create and broadcast presence
        let message = SyncMessage::Presence(shared::PresenceInfo {
            user_id,
            user_name: "Test User".to_string(),
            doc_id,
            cursor_position,
            is_typing,
            last_active: chrono::Utc::now(),
        });
        let count = sender.send(message.clone());

        prop_assert_eq!(count, num_receivers);

        // Verify presence for each receiver
        for receiver in &mut receivers {
            let received = receiver.recv_timeout(Duration::from_millis(100));
            prop_assert!(received.is_ok());

            if let SyncMessage::Presence(presence) = received.unwrap() {
                prop_assert_eq!(presence.user_id, user_id);
                prop_assert_eq!(presence.is_typing, is_typing);
                prop_assert_eq!(presence.cursor_position, cursor_position);
            } else {
                prop_assert!(false, "Expected Presence message");
            }
        }
    }
}

/// Test that multiple sequential broadcasts work correctly
///
/// Multiple broadcasts should each be delivered to all active subscribers.
proptest! {
    #[test]
    fn test_sequential_broadcasts(
        num_broadcasts in 5..20usize,
        num_receivers in 1..5usize,
    ) {
        let state = create_test_state_with_channels(1);
        let doc_id = Uuid::new_v4();

        // Create channel and subscribe
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        let mut receivers: Vec<broadcast::Receiver<SyncMessage>> = (0..num_receivers)
            .map(|_| sender.subscribe())
            .collect();

        // Send multiple broadcasts
        for i in 0..num_broadcasts {
            let message = SyncMessage::Ack { sequence: i as u64 };
            let count = sender.send(message.clone());

            prop_assert_eq!(count, num_receivers);
        }

        // All receivers should get all messages
        for receiver in &mut receivers {
            for i in 0..num_broadcasts {
                let received = receiver.recv_timeout(Duration::from_millis(50));
                prop_assert!(received.is_ok(), "Receiver missed message {}", i);

                if let Ok(SyncMessage::Ack { sequence }) = received {
                    prop_assert_eq!(sequence, i as u64);
                }
            }
        }
    }
}

/// Test that channel removal works correctly
///
/// Removing a channel should prevent future broadcasts.
proptest! {
    #[test]
    fn test_channel_removal(
        _doc_id in doc_id_strategy(),
    ) {
        let state = create_test_state_with_channels(1);
        let doc_id = Uuid::new_v4();

        // Create channel
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        // Verify channel exists
        prop_assert!(state.doc_channels.get(&doc_id).is_some());

        // Remove channel
        let removed = state.doc_channels.remove(&doc_id);
        prop_assert!(removed.is_some());

        // Channel should no longer exist
        prop_assert!(state.doc_channels.get(&doc_id).is_none());

        // Broadcasting to removed channel should return error
        let message = SyncMessage::Ack { sequence: 1 };
        let result = sender.send(message);

        // Send should fail (no subscribers)
        prop_assert!(result.is_err());
    }
}

/// Test that subscriber count is accurate
///
/// The receiver count should accurately reflect the number of
/// active subscribers.
proptest! {
    #[test]
    fn test_subscriber_count_accuracy(
        initial_subscribers in 1..10usize,
        additional_subscribers in 0..5usize,
        disconnects in 0..5usize,
    ) {
        let state = create_test_state_with_channels(1);
        let doc_id = Uuid::new_v4();

        // Create channel
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        // Initial subscribers
        let mut receivers: Vec<_> = (0..initial_subscribers)
            .map(|_| sender.subscribe())
            .collect();

        prop_assert_eq!(sender.receiver_count(), initial_subscribers);

        // Add more subscribers
        for _ in 0..additional_subscribers {
            receivers.push(sender.subscribe());
        }

        let total = initial_subscribers + additional_subscribers;
        prop_assert_eq!(sender.receiver_count(), total);

        // Disconnect some
        let disconnect_count = disconnects.min(receivers.len());
        receivers.truncate(receivers.len() - disconnect_count);

        let expected = total - disconnect_count;
        prop_assert_eq!(sender.receiver_count(), expected);
    }
}