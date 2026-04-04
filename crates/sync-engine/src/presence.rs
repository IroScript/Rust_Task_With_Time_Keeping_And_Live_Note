//! Real-time presence system for collaborative document editing.
//!
//! This module manages user presence information including cursor positions
//! and typing indicators for real-time collaboration features.
//!
//! # Requirements
//! - 4.4: Remove user from presence maps on disconnect
//! - 8.1: Add user to presence map when connecting to document
//! - 8.2: Track cursor position and typing indicators
//! - 8.3: Broadcast presence updates to all users viewing document
//! - 8.4: Remove disconnected users within 5 seconds
//! - 8.5: Display cursor position and typing status

use shared::PresenceInfo;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Presence manager for handling real-time user presence in documents.
///
/// Manages presence maps per document and broadcasts presence updates
/// to all subscribers of a document.
#[derive(Clone, Debug)]
pub struct PresenceManager {
    /// Document-specific presence maps (doc_id -> Vec<PresenceInfo>)
    presence: Arc<DashMap<Uuid, Vec<PresenceInfo>>>,
    /// Document-specific broadcast channels for presence updates
    doc_channels: Arc<DashMap<Uuid, broadcast::Sender<PresenceInfo>>>,
}

impl PresenceManager {
    /// Create a new PresenceManager with empty maps.
    #[must_use]
    pub fn new() -> Self {
        Self {
            presence: Arc::new(DashMap::new()),
            doc_channels: Arc::new(DashMap::new()),
        }
    }

    /// Get or create the broadcast channel for a document.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    ///
    /// # Returns
    /// The broadcast sender for the document's presence updates
    fn get_or_create_channel(&self, doc_id: Uuid) -> broadcast::Sender<PresenceInfo> {
        self.doc_channels
            .entry(doc_id)
            .or_insert_with(|| {
                // Create channel with reasonable buffer size for presence updates
                broadcast::Sender::new(256)
            })
            .clone()
    }

    /// Add a user to the presence map for a document.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `user_id` - The user's ID
    /// * `user_name` - The user's display name
    ///
    /// # Returns
    /// The initial PresenceInfo for the user
    ///
    /// # Requirements
    /// - 8.1: Add user to presence map when connecting to document
    pub fn add_user_to_document(
        &self,
        doc_id: Uuid,
        user_id: Uuid,
        user_name: String,
    ) -> PresenceInfo {
        let presence_info = PresenceInfo {
            user_id,
            user_name,
            doc_id,
            cursor_position: None,
            is_typing: false,
            last_active: chrono::Utc::now(),
        };

        let mut doc_presence = self.presence.entry(doc_id).or_insert_with(Vec::new);
        
        // Remove existing presence for this user if present (re-connection case)
        doc_presence.retain(|p| p.user_id != user_id);
        doc_presence.push(presence_info.clone());

        tracing::debug!(
            user_id = %user_id,
            doc_id = %doc_id,
            "User added to document presence"
        );

        presence_info
    }

    /// Update a user's presence timestamp and activity status.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `user_id` - The user's ID
    /// * `cursor_position` - Optional cursor position
    /// * `is_typing` - Whether the user is typing
    ///
    /// # Requirements
    /// - 8.2: Track cursor position and typing indicators
    pub fn update_presence(
        &self,
        doc_id: Uuid,
        user_id: Uuid,
        cursor_position: Option<usize>,
        is_typing: bool,
    ) -> Option<PresenceInfo> {
        let now = chrono::Utc::now();

        if let Some(mut doc_presence) = self.presence.get_mut(&doc_id) {
            for presence in doc_presence.iter_mut() {
                if presence.user_id == user_id {
                    presence.cursor_position = cursor_position;
                    presence.is_typing = is_typing;
                    presence.last_active = now;

                    tracing::debug!(
                        user_id = %user_id,
                        doc_id = %doc_id,
                        cursor_position = ?cursor_position,
                        is_typing,
                        "Presence updated"
                    );

                    return Some(presence.clone());
                }
            }
        }

        None
    }

    /// Update only the presence timestamp for a user.
    ///
    /// This is a convenience method for activity heartbeats that don't
    /// change cursor position or typing status.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `user_id` - The user's ID
    ///
    /// # Returns
    /// true if the presence was updated, false if user not found
    ///
    /// # Requirements
    /// - 8.2: Track user activity
    pub fn update_presence_timestamp(&self, doc_id: Uuid, user_id: Uuid) -> bool {
        let now = chrono::Utc::now();

        if let Some(mut doc_presence) = self.presence.get_mut(&doc_id) {
            for presence in doc_presence.iter_mut() {
                if presence.user_id == user_id {
                    presence.last_active = now;
                    return true;
                }
            }
        }

        false
    }

    /// Broadcast a presence update to all subscribers of a document.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `presence_info` - The presence information to broadcast
    ///
    /// # Returns
    /// Result indicating success or number of receivers if error
    ///
    /// # Requirements
    /// - 8.3: Broadcast presence updates to all users viewing document
    pub async fn broadcast_presence(
        &self,
        doc_id: Uuid,
        presence_info: PresenceInfo,
    ) -> Result<usize, broadcast::error::SendError<PresenceInfo>> {
        let sender = self.get_or_create_channel(doc_id);
        sender.send(presence_info.clone())?;

        tracing::debug!(
            user_id = %presence_info.user_id,
            doc_id = %doc_id,
            "Presence broadcast sent"
        );

        Ok(sender.receiver_count())
    }

    /// Subscribe to presence updates for a document.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    ///
    /// # Returns
    /// Receiver for presence updates
    pub fn subscribe_to_presence(&self, doc_id: Uuid) -> broadcast::Receiver<PresenceInfo> {
        let sender = self.get_or_create_channel(doc_id);
        sender.subscribe()
    }

    /// Get all presence information for a document.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    ///
    /// # Returns
    /// Vector of all PresenceInfo for users viewing the document
    pub fn get_document_presence(&self, doc_id: Uuid) -> Vec<PresenceInfo> {
        self.presence
            .get(&doc_id)
            .map(|ref_map| ref_map.value().clone())
            .unwrap_or_default()
    }

    /// Get presence for a specific user in a document.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `user_id` - The user's ID
    ///
    /// # Returns
    /// Some(PresenceInfo) if found, None otherwise
    pub fn get_user_presence(&self, doc_id: Uuid, user_id: Uuid) -> Option<PresenceInfo> {
        self.presence
            .get(&doc_id)
            .and_then(|ref_map| {
                ref_map.iter().find(|p| p.user_id == user_id).cloned()
            })
    }

    /// Remove a user from all presence maps (cleanup on disconnect).
    ///
    /// # Arguments
    /// * `user_id` - The user's ID
    ///
    /// # Returns
    /// Vector of document IDs the user was removed from
    ///
    /// # Requirements
    /// - 4.4: Remove user from presence maps on disconnect
    /// - 8.4: Remove disconnected users within 5 seconds
    pub fn remove_user_from_all_documents(&self, user_id: Uuid) -> Vec<Uuid> {
        let mut removed_from = Vec::new();

        for mut ref_map in self.presence.iter_mut() {
            let doc_id = *ref_map.key();
            let presence_list = ref_map.value_mut();
            
            let original_len = presence_list.len();
            presence_list.retain(|p| p.user_id != user_id);
            
            if presence_list.len() < original_len {
                removed_from.push(doc_id);
                tracing::debug!(
                    user_id = %user_id,
                    doc_id = %doc_id,
                    "User removed from document presence"
                );
            }
        }

        removed_from
    }

    /// Remove a user from a specific document's presence map.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `user_id` - The user's ID
    ///
    /// # Returns
    /// true if the user was removed, false if not found
    pub fn remove_user_from_document(&self, doc_id: Uuid, user_id: Uuid) -> bool {
        if let Some(mut ref_map) = self.presence.get_mut(&doc_id) {
            let presence_list = ref_map.value_mut();
            let original_len = presence_list.len();
            presence_list.retain(|p| p.user_id != user_id);
            
            if presence_list.len() < original_len {
                tracing::debug!(
                    user_id = %user_id,
                    doc_id = %doc_id,
                    "User removed from document presence"
                );
                return true;
            }
        }

        false
    }

    /// Clean up stale presence entries (users inactive for too long).
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `timeout_seconds` - Inactivity threshold in seconds
    ///
    /// # Returns
    /// Number of stale entries removed
    pub fn cleanup_stale_presence(&self, doc_id: Uuid, timeout_seconds: u64) -> usize {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(timeout_seconds as i64);

        if let Some(mut ref_map) = self.presence.get_mut(&doc_id) {
            let presence_list = ref_map.value_mut();
            let original_len = presence_list.len();
            presence_list.retain(|p| p.last_active >= cutoff);
            return original_len - presence_list.len();
        }

        0
    }

    /// Get the count of users viewing a document.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    ///
    /// # Returns
    /// Number of users in the presence map
    pub fn get_document_user_count(&self, doc_id: Uuid) -> usize {
        self.presence
            .get(&doc_id)
            .map(|ref_map| ref_map.len())
            .unwrap_or(0)
    }

    /// Check if a user is present in a document.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `user_id` - The user's ID
    ///
    /// # Returns
    /// true if the user is in the presence map
    pub fn is_user_present(&self, doc_id: Uuid, user_id: Uuid) -> bool {
        self.presence
            .get(&doc_id)
            .map(|ref_map| ref_map.iter().any(|p| p.user_id == user_id))
            .unwrap_or(false)
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn generate_test_uuid() -> Uuid {
        Uuid::new_v4()
    }

    #[tokio::test]
    async fn test_add_user_to_document() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        let presence = manager.add_user_to_document(doc_id, user_id, "Test User".to_string());

        assert_eq!(presence.user_id, user_id);
        assert_eq!(presence.user_name, "Test User");
        assert_eq!(presence.doc_id, doc_id);
        assert!(!presence.is_typing);
        assert!(presence.cursor_position.is_none());
    }

    #[tokio::test]
    async fn test_update_presence() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        manager.add_user_to_document(doc_id, user_id, "Test User".to_string());
        
        let updated = manager.update_presence(doc_id, user_id, Some(100), true);

        assert!(updated.is_some());
        let presence = updated.unwrap();
        assert_eq!(presence.cursor_position, Some(100));
        assert!(presence.is_typing);
    }

    #[tokio::test]
    async fn test_update_presence_timestamp() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        manager.add_user_to_document(doc_id, user_id, "Test User".to_string());
        
        let result = manager.update_presence_timestamp(doc_id, user_id);
        assert!(result);
    }

    #[tokio::test]
    async fn test_broadcast_presence() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        let presence = PresenceInfo {
            user_id,
            user_name: "Test User".to_string(),
            doc_id,
            cursor_position: Some(50),
            is_typing: true,
            last_active: chrono::Utc::now(),
        };

        let result = manager.broadcast_presence(doc_id, presence).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_document_presence() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user1_id = generate_test_uuid();
        let user2_id = generate_test_uuid();

        manager.add_user_to_document(doc_id, user1_id, "User 1".to_string());
        manager.add_user_to_document(doc_id, user2_id, "User 2".to_string());

        let presence = manager.get_document_presence(doc_id);
        assert_eq!(presence.len(), 2);
    }

    #[tokio::test]
    async fn test_remove_user_from_document() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        manager.add_user_to_document(doc_id, user_id, "Test User".to_string());
        assert_eq!(manager.get_document_user_count(doc_id), 1);

        let removed = manager.remove_user_from_document(doc_id, user_id);
        assert!(removed);
        assert_eq!(manager.get_document_user_count(doc_id), 0);
    }

    #[tokio::test]
    async fn test_remove_user_from_all_documents() {
        let manager = PresenceManager::new();
        let doc1_id = generate_test_uuid();
        let doc2_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        manager.add_user_to_document(doc1_id, user_id, "Test User".to_string());
        manager.add_user_to_document(doc2_id, user_id, "Test User".to_string());

        let removed_from = manager.remove_user_from_all_documents(user_id);
        assert_eq!(removed_from.len(), 2);
        assert_eq!(manager.get_document_user_count(doc1_id), 0);
        assert_eq!(manager.get_document_user_count(doc2_id), 0);
    }

    #[tokio::test]
    async fn test_is_user_present() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();
        let other_user_id = generate_test_uuid();

        manager.add_user_to_document(doc_id, user_id, "Test User".to_string());

        assert!(manager.is_user_present(doc_id, user_id));
        assert!(!manager.is_user_present(doc_id, other_user_id));
    }

    #[tokio::test]
    async fn test_get_user_presence() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        manager.add_user_to_document(doc_id, user_id, "Test User".to_string());
        manager.update_presence(doc_id, user_id, Some(200), false);

        let presence = manager.get_user_presence(doc_id, user_id);
        assert!(presence.is_some());
        assert_eq!(presence.unwrap().cursor_position, Some(200));
    }

    #[tokio::test]
    async fn test_cleanup_stale_presence() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        manager.add_user_to_document(doc_id, user_id, "Test User".to_string());
        assert_eq!(manager.get_document_user_count(doc_id), 1);

        // Clean up with 0 second timeout (all entries are stale)
        let removed = manager.cleanup_stale_presence(doc_id, 0);
        assert_eq!(removed, 1);
        assert_eq!(manager.get_document_user_count(doc_id), 0);
    }

    #[tokio::test]
    async fn test_subscribe_to_presence() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();

        let _receiver = manager.subscribe_to_presence(doc_id);
        // Receiver was created successfully
    }

    #[tokio::test]
    async fn test_reconnection_handling() {
        let manager = PresenceManager::new();
        let doc_id = generate_test_uuid();
        let user_id = generate_test_uuid();

        // First connection
        manager.add_user_to_document(doc_id, user_id, "Test User".to_string());
        assert_eq!(manager.get_document_user_count(doc_id), 1);

        // Reconnection (should not duplicate)
        manager.add_user_to_document(doc_id, user_id, "Test User".to_string());
        assert_eq!(manager.get_document_user_count(doc_id), 1);
    }
}