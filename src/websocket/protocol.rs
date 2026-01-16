use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::structs::{ChatMessage, PubUser};

/// Messages sent from client to server
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Send a text message to a chat
    SendMessage {
        chat_id: u64,
        message: String,
        reply_to_id: Option<u64>,
    },
    /// User started typing in a chat
    TypingStart {
        chat_id: u64,
    },
    /// User stopped typing in a chat
    TypingStop {
        chat_id: u64,
    },
    /// Mark a message as read
    MarkAsRead {
        message_id: u64,
    },
    /// Join a chat room (to start receiving messages)
    JoinChat {
        chat_id: u64,
    },
    /// Leave a chat room
    LeaveChat {
        chat_id: u64,
    },
    /// Ping to keep connection alive
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// New message received in a chat
    NewMessage {
        chat_id: u64,
        message: ChatMessage,
        sender: PubUser,
    },
    /// A message was read by a user
    MessageRead {
        message_id: u64,
        reader_id: u64,
        read_at: DateTime<Utc>,
    },
    /// User is typing in a chat
    UserTyping {
        chat_id: u64,
        user_id: u64,
        user_name: String,
    },
    /// User stopped typing in a chat
    UserStoppedTyping {
        chat_id: u64,
        user_id: u64,
    },
    /// User came online
    UserOnline {
        user_id: u64,
    },
    /// User went offline
    UserOffline {
        user_id: u64,
    },
    /// Error occurred
    Error {
        message: String,
    },
    /// Pong response to Ping
    Pong,
}

impl ServerMessage {
    /// Create a new error message
    pub fn error(msg: impl Into<String>) -> Self {
        ServerMessage::Error {
            message: msg.into(),
        }
    }
}
