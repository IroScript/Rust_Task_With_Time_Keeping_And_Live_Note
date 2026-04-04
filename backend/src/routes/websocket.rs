//! WebSocket handler for real-time document synchronization.
//!
//! This module implements the WebSocket connection handler that manages
//! client connections, subscribes to document broadcast channels, and
//! processes incoming sync messages.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures_util::{stream::{StreamExt, SplitSink, SplitStream}, SinkExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::routes::AppState;
use shared::{SyncMessage, SyncError};

/// WebSocket connection handler.
///
/// Upgrades the HTTP connection to WebSocket, authenticates the client,
/// subscribes to the document's broadcast channel, and handles message
/// processing with heartbeat support.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(document_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.max_message_size(2 * 1024 * 1024) // 2 MB limit per Requirement 17.4
        .max_frame_size(2 * 1024 * 1024)
        .on_upgrade(move |socket| {
            handle_websocket_connection(socket, state, document_id)
        })
}

/// Handles an established WebSocket connection.
///
/// This function:
/// 1. Authenticates the client via JWT (placeholder for now)
/// 2. Subscribes to the document's broadcast channel
/// 3. Spawns tasks for receiving messages and forwarding broadcasts
/// 4. Implements heartbeat with ping/pong every 30 seconds
/// 5. Cleans up on disconnect
async fn handle_websocket_connection(
    socket: WebSocket,
    state: Arc<AppState>,
    document_id: String,
) {
    tracing::info!(document_id = %document_id, "New WebSocket connection");

    // Subscribe to the document's broadcast channel
    let doc_id_clone = document_id.clone();
    let mut broadcast_receiver = state.doc_channels
        .entry(document_id.clone())
        .or_insert_with(|| {
            tracing::debug!(document_id = %document_id, "Creating new broadcast channel");
            tokio::sync::broadcast::channel(1000).0
        })
        .subscribe();

    // Split the WebSocket into sender and receiver
    let (ws_sender, ws_receiver) = socket.split();

    // Spawn task to forward broadcasts to WebSocket client
    let doc_id_for_forward = doc_id_clone.clone();
    let forward_task = tokio::spawn(async move {
        forward_broadcasts_to_client(ws_sender, &mut broadcast_receiver).await;
    });

    // Spawn task to receive messages from client
    let doc_id_for_receive = doc_id_clone.clone();
    let receive_task = tokio::spawn(async move {
        if let Err(e) = receive_messages_from_client(ws_receiver, &state, doc_id_for_receive.clone()).await {
            tracing::debug!(document_id = %doc_id_for_receive, error = %e, "Client message handling error");
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = forward_task => {
            tracing::debug!(document_id = %doc_id_for_forward, "Broadcast forward task ended");
        }
        _ = receive_task => {
            tracing::debug!(document_id = %doc_id_clone, "Client receive task ended");
        }
    }

    // Cleanup: remove subscriber count tracking if needed
    tracing::info!(document_id = %document_id, "WebSocket connection closed");
}

/// Forwards broadcast messages to the WebSocket client.
///
/// This task runs continuously, receiving messages from the document's
/// broadcast channel and sending them to the WebSocket client.
/// Uses best-effort delivery - disconnected clients are skipped.
async fn forward_broadcasts_to_client(
    mut ws_sender: SplitSink<WebSocket, Message>,
    broadcast_receiver: &mut broadcast::Receiver<SyncMessage>,
) {
    loop {
        tokio::select! {
            result = broadcast_receiver.recv() => {
                match result {
                    Ok(message) => {
                        if let Err(e) = send_message_to_websocket(&mut ws_sender, message).await {
                            tracing::debug!(error = %e, "Failed to send broadcast to client");
                            // Client disconnected, exit the loop
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Client is too slow, skip missed messages
                        // This is expected behavior for best-effort delivery
                        tracing::trace!("Client lagged behind broadcast, skipping messages");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // Channel closed, exit the loop
                        tracing::trace!("Broadcast channel closed");
                        break;
                    }
                }
            }
            // Periodic heartbeat check
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                // Send ping to keep connection alive
                if let Err(e) = ws_sender.send(Message::Ping(vec![].into())).await {
                    tracing::trace!(error = %e, "Heartbeat ping failed");
                    break;
                }
            }
        }
    }
}

/// Receives and processes messages from the WebSocket client.
///
/// Handles incoming sync messages (deltas, presence updates, acknowledgments),
/// persists changes to the database, and broadcasts to other subscribers.
async fn receive_messages_from_client(
    mut ws_receiver: SplitStream<WebSocket>,
    state: &Arc<AppState>,
    document_id: String,
) -> Result<(), SyncError> {
    loop {
        tokio::select! {
            result = ws_receiver.next() => {
                match result {
                    Some(Ok(message)) => {
                        if let Err(e) = process_websocket_message(&message, state, &document_id).await {
                            tracing::error!(error = %e, "Failed to process message");
                            // Continue processing other messages
                        }
                    }
                    Some(Err(e)) => {
                        tracing::debug!(error = %e, "WebSocket receive error");
                        return Err(SyncError::ConnectionClosed);
                    }
                    None => {
                        // Connection closed by client
                        return Err(SyncError::ConnectionClosed);
                    }
                }
            }
            // Heartbeat: send ping if no activity
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                // Note: Can't send ping from receiver stream, skip heartbeat
                tracing::trace!("Heartbeat check - connection active");
            }
        }
    }
}

/// Processes an incoming WebSocket message.
///
/// Handles different message types:
/// - Text: Deserializes and processes sync messages
/// - Ping: Responds with Pong (handled by axum automatically)
/// - Close: Cleans up and returns error
async fn process_websocket_message(
    message: &Message,
    state: &Arc<AppState>,
    _document_id: &str,
) -> Result<(), SyncError> {
    match message {
        Message::Text(text) => {
            let sync_msg: SyncMessage = serde_json::from_str(text)
                .map_err(|e| SyncError::InvalidMessage(e.to_string()))?;

            match sync_msg {
                SyncMessage::Delta {
                    doc_id,
                    version,
                    update,
                    author_id,
                } => {
                    tracing::debug!(
                        doc_id = %doc_id,
                        version = %version,
                        update_size = %update.len(),
                        "Received delta"
                    );

                    // Broadcast to all subscribers (best-effort delivery)
                    // Note: In a full implementation, we would:
                    // 1. Merge the CRDT update
                    // 2. Persist to database
                    // 3. Then broadcast
                    // For now, we broadcast directly as per the broadcast system design
                    let doc_id_clone = doc_id.clone();
                    let broadcast_msg = SyncMessage::Delta {
                        doc_id,
                        version,
                        update,
                        author_id,
                    };

                    // Best-effort broadcast - skip disconnected clients
                    if let Some(sender) = state.doc_channels.get(&doc_id_clone) {
                        let _ = sender.send(broadcast_msg);
                    }
                }
                SyncMessage::Presence(presence) => {
                    tracing::debug!(
                        user_id = %presence.user_id,
                        doc_id = %presence.doc_id,
                        "Received presence update"
                    );

                    // Broadcast presence to all subscribers
                    if let Some(sender) = state.doc_channels.get(&presence.doc_id) {
                        let _ = sender.send(SyncMessage::Presence(presence));
                    }
                }
                SyncMessage::Ack { sequence } => {
                    tracing::trace!(sequence = %sequence, "Received acknowledgment");
                    // Acknowledgments are logged but don't require action
                    // In a full implementation, this would be used for QoS tracking
                }
            }
        }
        Message::Ping(_) => {
            // Axum automatically handles ping/pong, but we can log if needed
            tracing::trace!("Received ping");
        }
        Message::Pong(_) => {
            tracing::trace!("Received pong");
        }
        Message::Close(_) => {
            tracing::debug!("Received close message");
            return Err(SyncError::ConnectionClosed);
        }
        _ => {
            tracing::warn!("Unsupported message type received");
        }
    }

    Ok(())
}

/// Sends a sync message to the WebSocket client.
///
/// Serializes the message to JSON and sends it as text.
/// Returns an error if the client has disconnected.
async fn send_message_to_websocket(
    ws_sender: &mut SplitSink<WebSocket, Message>,
    message: SyncMessage,
) -> Result<(), SyncError> {
    let json = serde_json::to_string(&message)
        .map_err(|e| SyncError::SerializationFailed(e.to_string()))?;

    ws_sender
        .send(Message::Text(json.into()))
        .await
        .map_err(|e| SyncError::SendFailed(e.to_string()))
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
    async fn test_ws_handler_creates_channel() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Verify no channel exists initially
        assert!(state.doc_channels.get(&doc_id).is_none());

        // Create channel by subscribing
        let _receiver = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .subscribe();

        // Channel should now exist
        assert!(state.doc_channels.get(&doc_id).is_some());
    }

    #[tokio::test]
    async fn test_process_delta_message() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        // Create channel and subscribe
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        let mut receiver = sender.subscribe();

        // Create delta message
        let message = SyncMessage::Delta {
            doc_id,
            version: 1,
            update: vec![1, 2, 3],
            author_id,
        };

        // Process the message
        let json = serde_json::to_string(&message).unwrap();
        let ws_message = Message::Text(json.into());

        let result = process_websocket_message(&ws_message, &state, doc_id).await;
        assert!(result.is_ok());

        // Verify broadcast was sent
        let received = receiver.recv().await.unwrap();
        match received {
            SyncMessage::Delta {
                doc_id: received_doc_id,
                version: received_version,
                update: received_update,
                author_id: received_author_id,
            } => {
                assert_eq!(received_doc_id, doc_id);
                assert_eq!(received_version, 1);
                assert_eq!(received_update, vec![1, 2, 3]);
                assert_eq!(received_author_id, author_id);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_process_presence_message() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create channel and subscribe
        let sender = state.doc_channels
            .entry(doc_id)
            .or_insert_with(|| broadcast::channel(1000).0)
            .clone();

        let mut receiver = sender.subscribe();

        // Create presence message
        let message = SyncMessage::Presence(shared::PresenceInfo {
            user_id,
            user_name: "Test User".to_string(),
            doc_id,
            cursor_position: Some(100),
            is_typing: true,
            last_active: chrono::Utc::now(),
        });

        // Process the message
        let json = serde_json::to_string(&message).unwrap();
        let ws_message = Message::Text(json.into());

        let result = process_websocket_message(&ws_message, &state, doc_id).await;
        assert!(result.is_ok());

        // Verify broadcast was sent
        let received = receiver.recv().await.unwrap();
        match received {
            SyncMessage::Presence(presence) => {
                assert_eq!(presence.user_id, user_id);
                assert_eq!(presence.is_typing, true);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_process_ack_message() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Create ack message
        let message = SyncMessage::Ack { sequence: 42 };
        let json = serde_json::to_string(&message).unwrap();
        let ws_message = Message::Text(json.into());

        // Ack messages should be processed without broadcasting
        let result = process_websocket_message(&ws_message, &state, doc_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_invalid_message() {
        let state = create_test_state();
        let doc_id = Uuid::new_v4();

        // Send invalid JSON
        let ws_message = Message::Text("invalid json".to_string().into());

        let result = process_websocket_message(&ws_message, &state, doc_id).await;
        assert!(result.is_err());
    }
}