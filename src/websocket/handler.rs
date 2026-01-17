use actix_web::{get, web, HttpRequest, HttpResponse, Error};
use actix_ws::Message;
use futures_util::StreamExt;
use sqlx::MySqlPool;
use chrono::Utc;

use crate::jwt::validate;
use crate::structs::{ChatMessage, PubUser};
use super::manager::ChatConnectionManager;
use super::protocol::{ClientMessage, ServerMessage};
use sqlx::FromRow;

/// Helper struct for joined query results
#[derive(FromRow)]
struct MessageWithSender {
    // ChatMessage fields
    id: u64,
    chat_id: u64,
    sender_id: u64,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    type_message: String,
    message: String,
    file_path: Option<String>,
    file_name: Option<String>,
    file_size: Option<u32>,
    is_deleted: bool,
    reply_to_id: Option<u64>,
    // PubUser fields (with sender_ prefix)
    sender_user_id: u64,
    sender_email: String,
    sender_photo: Option<String>,
    sender_course_id: Option<u64>,
    sender_full_name: Option<String>,
}

/// WebSocket endpoint for chat
#[get("/api/v1/ws/chat/")]
pub async fn chat_websocket(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<ChatConnectionManager>,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, Error> {
    log::info!("[WS DEBUG] New WebSocket connection request from {:?}", req.peer_addr());
    log::debug!("[WS DEBUG] Request headers: {:?}", req.headers());

    // Extract and validate JWT from cookie
    let cookie = match req.cookie("jwt") {
        Some(c) => {
            log::debug!("[WS DEBUG] JWT cookie found");
            c
        },
        None => {
            log::warn!("[WS DEBUG] WebSocket rejected: No JWT cookie found");
            log::debug!("[WS DEBUG] Available cookies: {:?}", req.cookies());
            return Ok(HttpResponse::Unauthorized().body("Authentication required"));
        }
    };

    let token = match validate(cookie.value()) {
        Ok(t) => {
            log::debug!("[WS DEBUG] JWT validated successfully");
            t
        },
        Err(e) => {
            log::warn!("[WS DEBUG] WebSocket rejected: Invalid JWT - {}", e);
            return Ok(HttpResponse::Unauthorized().body("Invalid authentication token"));
        }
    };

    let user_id = token.claims.user.id;
    let user_role = token.claims.user.role.clone();

    log::info!("[WS DEBUG] WebSocket: User {} ({:?}) authenticated successfully", user_id, user_role);

    // Upgrade HTTP connection to WebSocket
    log::info!("[WS DEBUG] Upgrading HTTP connection to WebSocket for user {}", user_id);
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    log::info!("[WS DEBUG] WebSocket upgrade successful for user {}", user_id);

    // Register connection in manager
    manager.connect(user_id, session.clone());
    log::info!("[WS DEBUG] User {} registered in connection manager", user_id);

    // Spawn a task to handle incoming messages
    let manager_clone = manager.get_ref().clone();
    let pool_clone = pool.get_ref().clone();

    actix_web::rt::spawn(async move {
        log::info!("[WS DEBUG] Message handler spawned for user {}", user_id);
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    log::info!("[WS DEBUG] Received text message from user {}: {}", user_id, text);

                    // Parse the client message
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(client_msg) => {
                            handle_client_message(
                                user_id,
                                client_msg,
                                &manager_clone,
                                &pool_clone,
                                &mut session,
                            )
                            .await;
                        }
                        Err(e) => {
                            log::warn!("Failed to parse message from user {}: {}", user_id, e);
                            let error_msg = ServerMessage::error("Invalid message format");
                            if let Ok(text) = serde_json::to_string(&error_msg) {
                                let _ = session.text(text).await;
                            }
                        }
                    }
                }
                Message::Ping(bytes) => {
                    log::debug!("Received ping from user {}", user_id);
                    let _ = session.pong(&bytes).await;
                }
                Message::Pong(_) => {
                    log::debug!("Received pong from user {}", user_id);
                }
                Message::Close(reason) => {
                    log::info!("[WS DEBUG] User {} closed WebSocket connection: {:?}", user_id, reason);
                    break;
                }
                _ => {
                    log::debug!("[WS DEBUG] Received other message type from user {}", user_id);
                }
            }
        }

        // Unregister connection when stream ends
        log::info!("[WS DEBUG] Disconnecting user {} from connection manager", user_id);
        manager_clone.disconnect(user_id);
        log::info!("[WS DEBUG] WebSocket handler ended for user {}", user_id);
    });

    Ok(response)
}

/// Handle different types of client messages
async fn handle_client_message(
    user_id: u64,
    message: ClientMessage,
    manager: &ChatConnectionManager,
    pool: &MySqlPool,
    session: &mut actix_ws::Session,
) {
    match message {
        ClientMessage::SendMessage { chat_id, message: msg_text, reply_to_id } => {
            handle_send_message(user_id, chat_id, msg_text, reply_to_id, manager, pool).await;
        }

        ClientMessage::TypingStart { chat_id } => {
            handle_typing_start(user_id, chat_id, manager, pool).await;
        }

        ClientMessage::TypingStop { chat_id } => {
            handle_typing_stop(user_id, chat_id, manager, pool).await;
        }

        ClientMessage::MarkAsRead { message_id } => {
            handle_mark_as_read(user_id, message_id, manager, pool).await;
        }

        ClientMessage::JoinChat { chat_id } => {
            log::info!("User {} joined chat {}", user_id, chat_id);
            // Could implement room-based routing here if needed
        }

        ClientMessage::LeaveChat { chat_id } => {
            log::info!("User {} left chat {}", user_id, chat_id);
            // Could implement room-based routing here if needed
        }

        ClientMessage::Ping => {
            let pong_msg = ServerMessage::Pong;
            if let Ok(text) = serde_json::to_string(&pong_msg) {
                let _ = session.text(text).await;
            }
        }
    }
}

/// Handle sending a new message
async fn handle_send_message(
    sender_id: u64,
    chat_id: u64,
    message: String,
    reply_to_id: Option<u64>,
    manager: &ChatConnectionManager,
    pool: &MySqlPool,
) {
    log::info!("[WS DEBUG] handle_send_message: user={}, chat={}, msg_len={}", sender_id, chat_id, message.len());

    // Verify user is a participant of the chat
    log::debug!("[WS DEBUG] Checking if user {} is participant of chat {}", sender_id, chat_id);
    let is_participant: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE user_id = ? AND chat_id = ?)",
    )
    .bind(sender_id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    {
        Ok(exists) => {
            log::debug!("[WS DEBUG] Participant check result: {}", exists);
            exists
        },
        Err(e) => {
            log::error!("[WS DEBUG] Failed to verify chat participant: {} - SQL Error: {}", sender_id, e);
            manager.send_to_user(
                sender_id,
                ServerMessage::error("Failed to verify chat access"),
            ).await;
            return;
        }
    };

    if !is_participant {
        log::warn!(
            "User {} attempted to send message to chat {} without access",
            sender_id,
            chat_id
        );
        manager.send_to_user(
            sender_id,
            ServerMessage::error("You are not a participant of this chat"),
        ).await;
        return;
    }

    // Insert message into database
    log::debug!("[WS DEBUG] Inserting message into database for chat {}", chat_id);
    let insert_result = sqlx::query(
        "INSERT INTO chat_messages (chat_id, sender_id, message, type_message, reply_to_id, created_at)
         VALUES (?, ?, ?, 'text', ?, NOW())"
    )
    .bind(chat_id)
    .bind(sender_id)
    .bind(&message)
    .bind(reply_to_id)
    .execute(pool)
    .await;

    let message_id = match insert_result {
        Ok(result) => {
            log::info!("[WS DEBUG] Message inserted with ID: {}", result.last_insert_id());
            result.last_insert_id()
        },
        Err(e) => {
            log::error!("[WS DEBUG] Failed to insert message: {} - SQL Error: {}", sender_id, e);
            manager.send_to_user(sender_id, ServerMessage::error("Failed to send message")).await;
            return;
        }
    };

    // Fetch the created message with sender info
    let query = "
        SELECT cm.id, cm.chat_id, cm.sender_id, cm.created_at, cm.updated_at,
               cm.type_message, cm.message, cm.file_path, cm.file_name, cm.file_size,
               cm.is_deleted, cm.reply_to_id,
               u.id as sender_user_id, u.email as sender_email, u.photo as sender_photo,
               u.course_id as sender_course_id, pd.full_name as sender_full_name
        FROM chat_messages cm
        JOIN users u ON cm.sender_id = u.id
        LEFT JOIN personal_data pd ON u.id = pd.user_id
        WHERE cm.id = ?
    ";

    log::debug!("[WS DEBUG] Fetching created message with sender info, message_id={}", message_id);
    match sqlx::query_as::<_, MessageWithSender>(query)
        .bind(message_id)
        .fetch_one(pool)
        .await
    {
        Ok(row) => {
            log::debug!("[WS DEBUG] Message fetched successfully, broadcasting to chat {}", chat_id);
            // Construct ChatMessage and PubUser from the joined query result
            let chat_message = ChatMessage {
                id: row.id,
                chat_id: row.chat_id,
                sender_id: row.sender_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                type_message: row.type_message,
                message: row.message,
                file_path: row.file_path,
                file_name: row.file_name,
                file_size: row.file_size,
                is_deleted: row.is_deleted,
                reply_to_id: row.reply_to_id,
            };

            let sender = PubUser {
                id: row.sender_user_id,
                email: row.sender_email,
                photo: row.sender_photo,
                course_id: row.sender_course_id,
                full_name: row.sender_full_name,
            };

            // Broadcast to all chat participants
            let server_msg = ServerMessage::NewMessage {
                chat_id,
                message: chat_message,
                sender,
            };

            manager.send_to_chat(chat_id, server_msg, pool, None).await;
        }
        Err(e) => {
            log::error!("Failed to fetch created message: {}", e);
        }
    }

    log::info!(
        "Message {} sent by user {} to chat {}",
        message_id,
        sender_id,
        chat_id
    );
}

/// Handle typing indicator start
async fn handle_typing_start(
    user_id: u64,
    chat_id: u64,
    manager: &ChatConnectionManager,
    pool: &MySqlPool,
) {

    // Verify user is a participant of the chat
    let is_participant: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE user_id = ? AND chat_id = ?)",
    )
    .bind(user_id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    {
        Ok(exists) => exists,
        Err(e) => {
            log::error!("Failed to verify chat participant: {}", e);
            manager.send_to_user(
                user_id,
                ServerMessage::error("Failed to verify chat access"),
            ).await;
            return;
        }
    };

    if !is_participant {
        log::warn!(
            "User {} attempted to send message to chat {} without access",
            user_id,
            chat_id
        );
        manager.send_to_user(
            user_id,
            ServerMessage::error("You are not a participant of this chat"),
        ).await;
        return;
    }
    // Insert or update typing indicator (expires in 5 seconds)
    let expires_at = Utc::now() + chrono::Duration::seconds(5);

    let _ = sqlx::query(
        "INSERT INTO typing_indicators (chat_id, user_id, started_at, expires_at)
         VALUES (?, ?, NOW(), ?)
         ON DUPLICATE KEY UPDATE started_at = NOW(), expires_at = ?"
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(expires_at)
    .bind(expires_at)
    .execute(pool)
    .await;

    // Get user's full name
    let user_name: String = sqlx::query_scalar(
        "SELECT COALESCE(pd.full_name, u.email) FROM users u
         LEFT JOIN personal_data pd ON u.id = pd.user_id WHERE u.id = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|_| format!("User {}", user_id));

    // Notify other participants
    let typing_msg = ServerMessage::UserTyping {
        chat_id,
        user_id,
        user_name,
    };

    manager.send_to_chat(chat_id, typing_msg, pool, Some(user_id)).await;
}

/// Handle typing indicator stop
async fn handle_typing_stop(
    user_id: u64,
    chat_id: u64,
    manager: &ChatConnectionManager,
    pool: &MySqlPool,
) {
    // Verify user is a participant of the chat
    let is_participant: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE user_id = ? AND chat_id = ?)",
    )
    .bind(user_id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    {
        Ok(exists) => exists,
        Err(e) => {
            log::error!("Failed to verify chat participant: {}", e);
            manager.send_to_user(
                user_id,
                ServerMessage::error("Failed to verify chat access"),
            ).await;
            return;
        }
    };

    if !is_participant {
        log::warn!(
            "User {} attempted to send message to chat {} without access",
            user_id,
            chat_id
        );
        manager.send_to_user(
            user_id,
            ServerMessage::error("You are not a participant of this chat"),
        ).await;
        return;
    }


    // Delete typing indicator
    let _ = sqlx::query("DELETE FROM typing_indicators WHERE user_id = ? AND chat_id = ?")
        .bind(user_id)
        .bind(chat_id)
        .execute(pool)
        .await;

    // Notify other participants
    let stopped_msg = ServerMessage::UserStoppedTyping { chat_id, user_id };

    manager.send_to_chat(chat_id, stopped_msg, pool, Some(user_id)).await;
}

/// Handle marking message as read
async fn handle_mark_as_read(
    user_id: u64,
    message_id: u64,
    manager: &ChatConnectionManager,
    pool: &MySqlPool,
) {
    //get the chat_id from message
        let chat_id: u64 = 
            sqlx::query_scalar("SELECT chat_id FROM chat_messages WHERE id = ?")
                .bind(message_id)
                .fetch_one(pool)
                .await
                .unwrap();
    
    // Verify user is a participant of the chat
    let is_participant: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE user_id = ? AND chat_id = ?)",
    )
    .bind(user_id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    {
        Ok(exists) => exists,
        Err(e) => {
            log::error!("Failed to verify chat participant: {}", e);
            manager.send_to_user(
                user_id,
                ServerMessage::error("Failed to verify chat access"),
            ).await;
            return;
        }
    };

    if !is_participant {
        log::warn!(
            "User {} attempted to send message to chat {} without access",
            user_id,
            chat_id
        );
        manager.send_to_user(
            user_id,
            ServerMessage::error("You are not a participant of this chat"),
        ).await;
        return;
    }

    // Insert read receipt (ignore duplicates)
    match sqlx::query(
        "INSERT INTO `reads` (message_id, reader_id, read_at)
         VALUES (?, ?, NOW())
         ON DUPLICATE KEY UPDATE read_at = NOW()"
    )
    .bind(message_id)
    .bind(user_id)
    .execute(pool)
    .await
    {
        Ok(_) => {

            let read_msg = ServerMessage::MessageRead {
                message_id,
                reader_id: user_id,
                read_at: Utc::now(),
            };

            manager.send_to_chat(chat_id, read_msg, pool, Some(user_id)).await;
        }
        Err(e) => {
            log::error!("Failed to mark message as read: {}", e);
        }
    }
}
