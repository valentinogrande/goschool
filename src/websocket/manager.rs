use actix_ws::Session;
use dashmap::DashMap;
use sqlx::MySqlPool;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::protocol::ServerMessage;

/// Manages all active WebSocket connections
#[derive(Clone)]
pub struct ChatConnectionManager {
    /// Map of user_id to their WebSocket session (wrapped in Arc<Mutex> for interior mutability)
    connections: Arc<DashMap<u64, Arc<Mutex<Session>>>>,
}

impl ChatConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
        }
    }

    /// Register a new connection for a user
    pub fn connect(&self, user_id: u64, session: Session) {
        log::info!("WebSocket: User {} connected", user_id);
        self.connections.insert(user_id, Arc::new(Mutex::new(session)));

        // Notify other users that this user is online
        let msg = ServerMessage::UserOnline { user_id };
        let manager = self.clone();
        tokio::spawn(async move {
            manager.broadcast_to_all(msg, Some(user_id)).await;
        });
    }

    /// Remove a user's connection
    pub fn disconnect(&self, user_id: u64) {
        log::info!("WebSocket: User {} disconnected", user_id);
        self.connections.remove(&user_id);

        // Notify other users that this user is offline
        let msg = ServerMessage::UserOffline { user_id };
        let manager = self.clone();
        tokio::spawn(async move {
            manager.broadcast_to_all(msg, Some(user_id)).await;
        });
    }

    /// Send a message to a specific user
    pub async fn send_to_user(&self, user_id: u64, message: ServerMessage) {
        if let Some(session_arc) = self.connections.get(&user_id) {
            if let Ok(text) = serde_json::to_string(&message) {
                let mut session = session_arc.lock().await;
                let _ = session.text(text);
            }
        }
    }

    /// Send a message to all participants in a chat
    pub async fn send_to_chat(
        &self,
        chat_id: u64,
        message: ServerMessage,
        pool: &MySqlPool,
        exclude_user_id: Option<u64>,
    ) {
        // Get all participants of the chat
        let query = "SELECT user_id FROM chat_participants WHERE chat_id = ?";

        match sqlx::query_scalar::<_, u64>(query)
            .bind(chat_id)
            .fetch_all(pool)
            .await
        {
            Ok(participant_ids) => {
                for user_id in participant_ids {
                    // Skip the excluded user (usually the sender)
                    if Some(user_id) == exclude_user_id {
                        continue;
                    }

                    self.send_to_user(user_id, message.clone()).await;
                }
            }
            Err(e) => {
                log::error!("Failed to fetch chat participants: {}", e);
            }
        }
    }

    /// Broadcast a message to all connected users
    pub async fn broadcast_to_all(&self, message: ServerMessage, exclude_user_id: Option<u64>) {
        let text = match serde_json::to_string(&message) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to serialize message: {}", e);
                return;
            }
        };

        for entry in self.connections.iter() {
            let user_id = *entry.key();

            // Skip the excluded user
            if Some(user_id) == exclude_user_id {
                continue;
            }

            let session_arc = entry.value().clone();
            let text_clone = text.clone();
            tokio::spawn(async move {
                let mut session = session_arc.lock().await;
                let _ = session.text(text_clone);
            });
        }
    }

    /// Get list of online users in a specific chat
    pub async fn get_online_users_in_chat(
        &self,
        chat_id: u64,
        pool: &MySqlPool,
    ) -> Vec<u64> {
        // Get all participants of the chat
        let query = "SELECT user_id FROM chat_participants WHERE chat_id = ?";

        let participant_ids = match sqlx::query_scalar::<_, u64>(query)
            .bind(chat_id)
            .fetch_all(pool)
            .await
        {
            Ok(ids) => ids,
            Err(e) => {
                log::error!("Failed to fetch chat participants: {}", e);
                return Vec::new();
            }
        };

        // Filter to only those who are online
        participant_ids
            .into_iter()
            .filter(|user_id| self.connections.contains_key(user_id))
            .collect()
    }

    /// Check if a user is currently online
    pub fn is_user_online(&self, user_id: u64) -> bool {
        self.connections.contains_key(&user_id)
    }

    /// Get count of active connections
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for ChatConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
