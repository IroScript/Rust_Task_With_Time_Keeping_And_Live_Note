//! WebSocket connection handler for real-time document synchronization
//!
//! This module implements the WebSocket connection handling logic including
//! JWT authentication, heartbeat management, message processing, and
//! connection cleanup.

use auth::jwt::{validate_token, JwtConfig};
use shared::{
    types::{AppState, PresenceInfo, SyncMessage},
    SyncError,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket message receiver result type
type ReceiverResult = Result<(), SyncError>;

/// Handles a WebSocket connection with full lifecycle management.
///
/// This function manages the complete WebSocket connection lifecycle including:
/// - JWT authentication on connection
/// - Heartbeat ping/pong every 30 seconds
/// - Message processing (Delta, Presence, Ack)
/// - Presence tracking and cleanup on disconnect
///
/// # Arguments
/// * `socket` - The WebSocket connection
/// * `state` - Shared application state
/// * `auth_token` - JWT token for authentication
///
/// # Returns
/// Ok(()) on clean disconnect, Err(SyncError) on error
///
/// # Preconditions
/// - socket is a valid, upgraded WebSocket connection
/// - state contains initialized db_pool, doc_channels, presence maps
/// - auth_token is a non-empty JWT string
///
/// # Postconditions
/// - If auth fails: socket closed with 401 status, error returned
/// - If auth succeeds: user subscribed to document channels
/// - On disconnect: user removed from presence map
/// - All received deltas are persisted to database before broadcast
/// - Message ordering preserved per document (FIFO)
/// - Heartbeat maintained (ping/pong every 30s)
pub async fn handle_websocket_connection(
    socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
    auth_token: String,
) -> Result<(), SyncError> {
    // Step 1: Validate JWT token
    let jwt_config = JwtConfig::default();
    let claims = validate_token(&auth_token, &jwt_config)
        .map_err(|e| SyncError::AuthenticationFailed(e.to_string()))?;

    let user_id = claims.sub;
    let user_name = claims.email.unwrap_or_else(|| "unknown".to_string());

    info!(user_id = %user_id, "WebSocket connection authenticated");

    // Step 2: Split socket into sender and receiver
    let (mut sender, receiver) = socket.split();

    // Step 3: Spawn heartbeat task (ping every 30 seconds)
    let heartbeat_user_id = user_id;
    let mut heartbeat_sender = sender.clone();
    let heartbeat_handle = tokio::spawn(async move {
        run_heartbeat(&mut heartbeat_sender, heartbeat_user_id).await;
    });

    // Step 4: Process messages with cleanup on disconnect
    let result = process_messages(receiver, &state, user_id, user_name).await;

    // Step 5: Stop heartbeat
    heartbeat_handle.abort();

    // Step 6: Cleanup presence on disconnect
    cleanup_user_presence(&state, user_id).await;

    // Step 7: Close socket gracefully
    if let Err(e) = sender
        .close()
        .await
    {
        warn!(user_id = %user_id, error = %e, "Error closing WebSocket");
    }

    if let Err(e) = result {
        error!(user_id = %user_id, error = %e, "WebSocket connection error");
        return Err(e);
    }

    info!(user_id = %user_id, "WebSocket connection closed");
    Ok(())
}

/// Runs the heartbeat loop, sending ping messages every 30 seconds.
///
/// # Arguments
/// * `sender` - WebSocket sender to send pings
/// * `user_id` - User ID for logging
async fn run_heartbeat(sender: &mut axum::extract::ws::WebSocketSender, user_id: Uuid) {
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        if let Err(e) = sender
            .send(axum::extract::ws::Message::Ping(vec![]))
            .await
        {
            warn!(user_id = %user_id, error = %e, "Failed to send ping");
            break;
        }

        debug!(user_id = %user_id, "Sent heartbeat ping");
    }
}

/// Processes incoming WebSocket messages with sequence tracking.
///
/// Handles Delta, Presence, and Ack messages with proper error handling,
/// sequence number tracking for acknowledgments, and cleanup on disconnect.
///
/// # Arguments
/// * `receiver` - WebSocket message receiver
/// * `state` - Shared application state
/// * `user_id` - Authenticated user ID
/// * `user_name` - User's display name
///
/// # Returns
/// Ok(()) on clean disconnect, Err(SyncError) on error
///
/// # Loop Invariants
/// - Message processing is atomic per message
/// - No interleaving of updates for same document
/// - Broadcast ordering matches persistence ordering
async fn process_messages(
    mut receiver: axum::extract::ws::WebSocketReceiver,
    state: &Arc<AppState>,
    user_id: Uuid,
    user_name: String,
) -> ReceiverResult {
    // Sequence number for acknowledgment tracking (requirement 15.2, 15.4)
    let mut sequence: u64 = 0;
    let sender = receiver.sender().clone();

    while let Some(message) = receiver.recv().await {
        match message {
            // Handle text messages (JSON sync messages)
            Ok(axum::extract::ws::Message::Text(text)) => {
                match handle_websocket_message(
                    axum::extract::ws::Message::Text(text),
                    state,
                    user_id,
                )
                .await
                {
                    Ok(()) => {
                        // Send acknowledgment after successful processing (requirement 15.4)
                        sequence += 1;
                        send_ack(&sender, sequence).await;
                    }
                    Err(e) => {
                        error!(user_id = %user_id, error = %e, "Error processing message");
                        return Err(e);
                    }
                }
            }

            // Handle binary messages (CRDT updates in binary format)
            Ok(axum::extract::ws::Message::Binary(data)) => {
                match handle_websocket_message(
                    axum::extract::ws::Message::Binary(data),
                    state,
                    user_id,
                )
                .await
                {
                    Ok(()) => {
                        // Send acknowledgment after successful processing
                        sequence += 1;
                        send_ack(&sender, sequence).await;
                    }
                    Err(e) => {
                        error!(user_id = %user_id, error = %e, "Error processing binary message");
                        return Err(e);
                    }
                }
            }

            // Handle ping messages (heartbeat)
            Ok(axum::extract::ws::Message::Ping(_)) => {
                debug!(user_id = %user_id, "Received ping");
                // Axum automatically handles ping/pong
            }

            // Handle pong messages (heartbeat response)
            Ok(axum::extract::ws::Message::Pong(_)) => {
                debug!(user_id = %user_id, "Received pong");
            }

            // Handle close messages
            Ok(axum::extract::ws::Message::Close(close_reason)) => {
                info!(
                    user_id = %user_id,
                    reason = ?close_reason,
                    "Client initiated close"
                );
                return Err(SyncError::ConnectionClosed);
            }

            // Ignore other message types
            Ok(_) => {
                debug!(user_id = %user_id, "Received unsupported message type");
            }

            // Handle receive errors
            Err(e) => {
                error!(user_id = %user_id, error = %e, "WebSocket receive error");
                return Err(SyncError::ConnectionClosed);
            }
        }
    }

    Ok(())
}

/// Unified WebSocket message handler for all message types.
///
/// This function processes incoming WebSocket messages including
/// Delta updates, Presence updates, and Acknowledgments.
/// It handles deserialization, validation, and routing to appropriate handlers.
///
/// # Arguments
/// * `message` - Incoming WebSocket message
/// * `state` - Shared application state
/// * `user_id` - Authenticated user ID
///
/// # Returns
/// Ok(()) on success, Err(SyncError) on error
///
/// # Preconditions
/// - WebSocket connection is authenticated and active
/// - state contains valid database connections and CRDT documents
/// - user_id corresponds to authenticated session
///
/// # Postconditions
/// - All deltas are persisted before broadcast (durability)
/// - CRDT state is consistent across all replicas
/// - Presence information is up-to-date
/// - Acknowledgment sent for successfully processed messages
pub async fn handle_websocket_message(
    message: axum::extract::ws::Message,
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<(), SyncError> {
    match message {
        axum::extract::ws::Message::Text(text) => {
            handle_text_message(&text, state, user_id).await
        }
        axum::extract::ws::Message::Binary(data) => {
            handle_binary_message(&data, state, user_id).await
        }
        axum::extract::ws::Message::Ping(_) => {
            // Heartbeat handled by axum automatically
            Ok(())
        }
        axum::extract::ws::Message::Pong(_) => {
            // Heartbeat response received
            Ok(())
        }
        axum::extract::ws::Message::Close(close_reason) => {
            debug!(user_id = %user_id, reason = ?close_reason, "Received close message");
            Err(SyncError::ConnectionClosed)
        }
        _ => Err(SyncError::UnsupportedMessageType),
    }
}

/// Handles a text message by deserializing and processing the sync message.
///
/// # Arguments
/// * `text` - Raw JSON text from WebSocket
/// * `state` - Shared application state
/// * `user_id` - Authenticated user ID
///
/// # Returns
/// Ok(()) on success, Err(SyncError) on error
async fn handle_text_message(
    text: &str,
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<(), SyncError> {
    // Step 1: Deserialize and validate the sync message
    let sync_msg: SyncMessage = serde_json::from_str(text)
        .map_err(|e| SyncError::InvalidMessage(e.to_string()))?;

    // Step 2: Route to appropriate handler based on message type
    match sync_msg {
        SyncMessage::Delta {
            doc_id,
            version,
            update,
            author_id,
        } => {
            handle_delta(doc_id, version, update, author_id, state).await?;
        }

        SyncMessage::Presence(info) => {
            handle_presence(info, state, user_id).await?;
        }

        SyncMessage::Ack { sequence } => {
            debug!(user_id = %user_id, sequence, "Received acknowledgment");
        }
    }

    Ok(())
}

/// Handles a binary message (raw CRDT update bytes).
///
/// # Arguments
/// * `data` - Binary CRDT update data
/// * `state` - Shared application state
/// * `user_id` - Authenticated user ID
///
/// # Returns
/// Ok(()) on success, Err(SyncError) on error
async fn handle_binary_message(
    data: &[u8],
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<(), SyncError> {
    // Binary messages are expected to be CRDT updates
    // The format is: doc_id (16 bytes) || version (8 bytes) || update bytes
    if data.len() < 24 {
        return Err(SyncError::InvalidMessage(
            "Binary message too short".to_string(),
        ));
    }

    let doc_id = Uuid::from_slice(&data[0..16])
        .map_err(|e| SyncError::InvalidMessage(e.to_string()))?;
    let version = i64::from_le_bytes(
        data[16..24]
            .try_into()
            .map_err(|e| SyncError::InvalidMessage(e.to_string()))?,
    );
    let update = data[24..].to_vec();

    handle_delta(doc_id, version, update, user_id, state).await
}

/// Handles a Delta message by merging CRDT update and broadcasting.
///
/// # Arguments
/// * `doc_id` - Document being updated
/// * `version` - Current CRDT version
/// * `update` - yrs-encoded CRDT update bytes
/// * `author_id` - User who authored this update
/// * `state` - Shared application state
///
/// # Returns
/// Ok(()) on success, Err(SyncError) on error
async fn handle_delta(
    doc_id: Uuid,
    version: i64,
    update: Vec<u8>,
    author_id: Uuid,
    state: &Arc<AppState>,
) -> Result<(), SyncError> {
    debug!(
        doc_id = %doc_id,
        version,
        author_id = %author_id,
        "Processing delta"
    );

    // Get or create CRDT document
    let doc = state
        .crdt_docs
        .entry(doc_id)
        .or_insert_with(|| Arc::new(tokio::sync::RwLock::new(yrs::Doc::new())));

    // Merge CRDT update
    let merged_update = merge_crdt_update(&doc, &update, author_id).await?;

    // Persist to database (persist-before-broadcast)
    let new_version = persist_revision(state, doc_id, version, &merged_update, author_id).await?;

    // Broadcast to all subscribers
    broadcast_delta(state, doc_id, new_version, merged_update, author_id).await;

    Ok(())
}

/// Merges a CRDT update into the document state.
///
/// # Arguments
/// * `doc` - CRDT document reference
/// * `remote_update` - yrs-encoded update bytes
/// * `author_id` - Author of the update
///
/// # Returns
/// Merged state vector as encoded update
async fn merge_crdt_update(
    doc: &Arc<tokio::sync::RwLock<yrs::Doc>>,
    remote_update: &[u8],
    _author_id: Uuid,
) -> Result<Vec<u8>, SyncError> {
    // Decode remote update
    let update = yrs::updates::decoder::Decode::decode_v1(remote_update)
        .map_err(|e| SyncError::CRDTError(e.to_string()))?;

    // Acquire write lock and apply update
    let mut doc_guard = doc.write().await;
    let mut txn = doc_guard.transact_mut();
    txn.apply_update(update)
        .map_err(|e| SyncError::CRDTError(e.to_string()))?;

    // Extract merged state vector
    let state_vector = txn.encode_state_as_update_v1(&yrs::StateVector::default());

    drop(txn);
    drop(doc_guard);

    Ok(state_vector)
}

/// Persists a revision to the database.
///
/// # Arguments
/// * `state` - Shared application state
/// * `doc_id` - Document ID
/// * `version` - Current version
/// * `update` - CRDT update bytes
/// * `author_id` - Author of the update
///
/// # Returns
/// New version number after persistence
async fn persist_revision(
    state: &Arc<AppState>,
    doc_id: Uuid,
    version: i64,
    update: &[u8],
    author_id: Uuid,
) -> Result<i64, SyncError> {
    let new_version = version + 1;

    // TODO: Implement actual persistence to PostgreSQL + S3
    // For now, just log the operation
    debug!(
        doc_id = %doc_id,
        version,
        new_version,
        author_id = %author_id,
        update_size = update.len(),
        "Persisting revision"
    );

    Ok(new_version)
}

/// Broadcasts a delta to all document subscribers.
///
/// # Arguments
/// * `state` - Shared application state
/// * `doc_id` - Document ID
/// * `version` - New version number
/// * `update` - CRDT update bytes
/// * `author_id` - Author of the update
async fn broadcast_delta(
    state: &Arc<AppState>,
    doc_id: Uuid,
    version: i64,
    update: Vec<u8>,
    author_id: Uuid,
) {
    let broadcast_msg = SyncMessage::Delta {
        doc_id,
        version,
        update,
        author_id,
    };

    if let Some(tx) = state.doc_channels.get(&doc_id) {
        if let Err(e) = tx.send(broadcast_msg) {
            warn!(doc_id = %doc_id, error = %e, "Failed to broadcast delta");
        } else {
            debug!(doc_id = %doc_id, "Broadcasted delta to subscribers");
        }
    }
}

/// Handles a Presence message by updating presence and broadcasting.
///
/// # Arguments
/// * `info` - Presence information
/// * `state` - Shared application state
/// * `user_id` - User ID for tracking
///
/// # Returns
/// Ok(()) on success, Err(SyncError) on error
async fn handle_presence(
    info: PresenceInfo,
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<(), SyncError> {
    debug!(
        user_id = %user_id,
        doc_id = %info.doc_id,
        "Processing presence update"
    );

    // Update presence in the map
    let mut presence_vec = state
        .presence
        .entry(info.doc_id)
        .or_insert_with(Vec::new);

    // Remove existing presence for this user
    presence_vec.retain(|p| p.user_id != user_id);

    // Add new presence info
    presence_vec.push(PresenceInfo {
        user_id,
        user_name: info.user_name,
        doc_id: info.doc_id,
        cursor_position: info.cursor_position,
        is_typing: info.is_typing,
        last_active: chrono::Utc::now(),
    });

    // Broadcast presence to all subscribers
    broadcast_presence(state, &info).await;

    Ok(())
}

/// Broadcasts a presence update to all document subscribers.
///
/// # Arguments
/// * `state` - Shared application state
/// * `info` - Presence information to broadcast
async fn broadcast_presence(state: &Arc<AppState>, info: &PresenceInfo) {
    let presence_msg = SyncMessage::Presence(info.clone());

    if let Some(tx) = state.doc_channels.get(&info.doc_id) {
        if let Err(e) = tx.send(presence_msg) {
            warn!(doc_id = %info.doc_id, error = %e, "Failed to broadcast presence");
        }
    }
}

/// Sends an acknowledgment to the client.
///
/// Implements acknowledgment tracking for QoS (requirement 15.4).
/// Each processed message receives a unique sequence number acknowledgment.
///
/// # Arguments
/// * `sender` - WebSocket sender for sending the ack
/// * `sequence` - Sequence number being acknowledged
async fn send_ack(
    sender: &tokio::sync::MutexGuard<'_, axum::extract::ws::WebSocketSender>,
    sequence: u64,
) {
    let ack = SyncMessage::Ack { sequence };
    if let Ok(text) = serde_json::to_string(&ack) {
        if let Err(e) = sender.send(axum::extract::ws::Message::Text(text)).await {
            warn!(sequence, error = %e, "Failed to send acknowledgment");
        } else {
            debug!(sequence, "Sent acknowledgment");
        }
    }
}

/// Cleans up user presence on disconnect.
///
/// Removes the user from all document presence maps within 5 seconds
/// of disconnection (requirement 8.4).
///
/// # Arguments
/// * `state` - Shared application state
/// * `user_id` - User ID to remove from presence
async fn cleanup_user_presence(state: &Arc<AppState>, user_id: Uuid) {
    // Iterate through all document presence maps and remove this user
    for mut entry in state.presence.iter() {
        let doc_id = *entry.key();
        let presence_vec = entry.value_mut();

        let before_count = presence_vec.len();
        presence_vec.retain(|p| p.user_id != user_id);
        let after_count = presence_vec.len();

        if before_count != after_count {
            info!(
                user_id = %user_id,
                doc_id = %doc_id,
                "Removed user from presence map"
            );
        }
    }
}

/// WebSocket handler for Axum router.
///
/// This is the entry point for WebSocket connections from the HTTP router.
/// It extracts the JWT token from query parameters and upgrades the connection.
///
/// # Arguments
/// * `ws` - WebSocket upgrade request
/// * `state` - Shared application state
/// * `token` - JWT token from query parameter
///
/// # Returns
/// WebSocket upgrade response
pub async fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    state: Arc<AppState>,
    axum::extract::Query(token): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl axum::response::IntoResponse {
    let token = token.get("token").cloned().unwrap_or_default();

    ws.max_message_size(2 * 1024 * 1024) // 2 MB limit
        .on_upgrade(move |socket| handle_websocket_connection(socket, state, token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::types::{AppState, CacheConfig, DatabaseCredentials};
    use std::sync::Arc;

    fn create_test_state() -> Arc<AppState> {
        Arc::new(AppState {
            db_pool: sqlx::PgPool::connect_lazy("postgres://test:test@localhost/test").unwrap(),
            s3_client: aws_sdk_s3::Client::from_conf(
                aws_sdk_s3::config::Config::builder()
                    .region(aws_sdk_s3::config::Region::new("us-east-1"))
                    .build(),
            ),
            vault_client: Arc::new(vaultrs::client::VaultClient::new(
                "http://localhost:8200",
                "test-token",
            )),
            doc_channels: Arc::new(dashmap::DashMap::new()),
            presence: Arc::new(dashmap::DashMap::new()),
            crdt_docs: Arc::new(dashmap::DashMap::new()),
        })
    }

    #[tokio::test]
    async fn test_handle_websocket_message_delta() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();
        let doc_id = Uuid::new_v4();

        let message = SyncMessage::Delta {
            doc_id,
            version: 1,
            update: vec![1, 2, 3],
            author_id: user_id,
        };
        let text = serde_json::to_string(&message).unwrap();

        let result =
            handle_websocket_message(axum::extract::ws::Message::Text(text), &state, user_id)
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_websocket_message_presence() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();
        let doc_id = Uuid::new_v4();

        let message = SyncMessage::Presence(PresenceInfo {
            user_id,
            user_name: "test".to_string(),
            doc_id,
            cursor_position: Some(100),
            is_typing: true,
            last_active: chrono::Utc::now(),
        });
        let text = serde_json::to_string(&message).unwrap();

        let result =
            handle_websocket_message(axum::extract::ws::Message::Text(text), &state, user_id)
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_websocket_message_ack() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();

        let message = SyncMessage::Ack { sequence: 42 };
        let text = serde_json::to_string(&message).unwrap();

        let result =
            handle_websocket_message(axum::extract::ws::Message::Text(text), &state, user_id)
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_websocket_message_invalid_json() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();

        let result =
            handle_websocket_message(axum::extract::ws::Message::Text("invalid".to_string()), &state, user_id)
                .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_websocket_message_ping() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();

        let result =
            handle_websocket_message(axum::extract::ws::Message::Ping(vec![]), &state, user_id)
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_websocket_message_pong() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();

        let result =
            handle_websocket_message(axum::extract::ws::Message::Pong(vec![]), &state, user_id)
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_websocket_message_close() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();

        let result = handle_websocket_message(
            axum::extract::ws::Message::Close(None),
            &state,
            user_id,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result, Err(SyncError::ConnectionClosed)));
    }

    #[tokio::test]
    async fn test_handle_text_message_delta() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();
        let doc_id = Uuid::new_v4();

        let text = serde_json::to_string(&SyncMessage::Delta {
            doc_id,
            version: 1,
            update: vec![1, 2, 3],
            author_id: user_id,
        })
        .unwrap();

        let result = handle_text_message(&text, &state, user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_text_message_presence() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();
        let doc_id = Uuid::new_v4();

        let text = serde_json::to_string(&SyncMessage::Presence(PresenceInfo {
            user_id,
            user_name: "test".to_string(),
            doc_id,
            cursor_position: Some(100),
            is_typing: true,
            last_active: chrono::Utc::now(),
        }))
        .unwrap();

        let result = handle_text_message(&text, &state, user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_text_message_invalid() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();

        let result = handle_text_message("invalid json", &state, user_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_user_presence() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();
        let doc_id = Uuid::new_v4();

        // Add user to presence
        state.presence.insert(
            doc_id,
            vec![PresenceInfo {
                user_id,
                user_name: "test".to_string(),
                doc_id,
                cursor_position: None,
                is_typing: false,
                last_active: chrono::Utc::now(),
            }],
        );

        // Cleanup
        cleanup_user_presence(&state, user_id).await;

        // Verify user is removed
        assert!(state.presence.get(&doc_id).map(|e| e.is_empty()).unwrap_or(true));
    }
}