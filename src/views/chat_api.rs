use actix_web::{get, post, put, delete, web, HttpRequest, HttpResponse, Responder};
use actix_multipart::Multipart;
use sqlx::MySqlPool;

use crate::jwt::validate;
use crate::structs::{NewChatRequest, SendMessageRequest, Chat, ChatMessage};
use crate::parse_multipart::parse_multipart;

/// GET /api/v1/chats/ - List user's chats with last message and unread count
#[get("/api/v1/chats/")]
pub async fn get_user_chats(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.user.id;

    // Get all chats for the user with last message and unread count
    let query = r#"
        SELECT DISTINCT
            c.id, c.name, c.photo, c.description, c.created_at, c.updated_at,
            c.chat_type, c.created_by,
            (SELECT cm.message FROM chat_messages cm
             WHERE cm.chat_id = c.id AND cm.is_deleted = FALSE
             ORDER BY cm.created_at DESC LIMIT 1) as last_message,
            (SELECT cm.created_at FROM chat_messages cm
             WHERE cm.chat_id = c.id AND cm.is_deleted = FALSE
             ORDER BY cm.created_at DESC LIMIT 1) as last_message_time,
            (SELECT COUNT(*) FROM chat_messages cm
             LEFT JOIN reads r ON r.message_id = cm.id AND r.reader_id = ?
             WHERE cm.chat_id = c.id AND cm.sender_id != ? AND r.id IS NULL
             AND cm.is_deleted = FALSE) as unread_count,
            FALSE as is_online
        FROM chats c
        JOIN chat_participants cp ON cp.chat_id = c.id
        WHERE cp.user_id = ?
        ORDER BY last_message_time DESC, c.updated_at DESC
    "#;

    match sqlx::query_as::<_, Chat>(query)
        .bind(user_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(chats) => HttpResponse::Ok().json(chats),
        Err(e) => {
            log::error!("Failed to fetch chats: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch chats")
        }
    }
}

/// POST /api/v1/chats/ - Create new chat (direct or group)
#[post("/api/v1/chats/")]
pub async fn create_chat(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    data: web::Json<NewChatRequest>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    // Validate chat type
    if data.chat_type != "direct" && data.chat_type != "group" {
        return HttpResponse::BadRequest().json("Invalid chat_type. Must be 'direct' or 'group'");
    }

    // Validate participants
    if data.participant_ids.is_empty() {
        return HttpResponse::BadRequest().json("At least one participant is required");
    }

    // For direct chats, should have exactly 1 other participant
    if data.chat_type == "direct" && data.participant_ids.len() != 1 {
        return HttpResponse::BadRequest().json("Direct chats must have exactly one other participant");
    }

    // For group chats, name is required
    if data.chat_type == "group" && data.name.is_none() {
        return HttpResponse::BadRequest().json("Group chats must have a name");
    }

    // Check if direct chat already exists
    if data.chat_type == "direct" {
        let other_user_id = data.participant_ids[0];

        let existing_chat: Option<u64> = sqlx::query_scalar(
            r#"
            SELECT c.id FROM chats c
            WHERE c.chat_type = 'direct'
            AND EXISTS (SELECT 1 FROM chat_participants WHERE chat_id = c.id AND user_id = ?)
            AND EXISTS (SELECT 1 FROM chat_participants WHERE chat_id = c.id AND user_id = ?)
            AND (SELECT COUNT(*) FROM chat_participants WHERE chat_id = c.id) = 2
            "#
        )
        .bind(user.id)
        .bind(other_user_id)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap_or(None);

        if let Some(chat_id) = existing_chat {
            return HttpResponse::Ok().json(serde_json::json!({ "chat_id": chat_id, "existed": true }));
        }
    }

    // Verify user can chat with all participants
    for &participant_id in &data.participant_ids {
        match user.can_chat_with(pool.get_ref(), participant_id).await {
            Ok(true) => {},
            Ok(false) => {
                return HttpResponse::Forbidden().json(format!("You cannot chat with user {}", participant_id));
            },
            Err(e) => {
                log::error!("Failed to verify chat authorization: {}", e);
                return HttpResponse::InternalServerError().json("Failed to verify authorization");
            }
        }
    }

    // Create chat name for direct chats
    let chat_name = if data.chat_type == "direct" {
        // For direct chats, use the other user's name
        let other_user_id = data.participant_ids[0];
        sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(pd.full_name, u.email) FROM users u
             LEFT JOIN personal_data pd ON u.id = pd.user_id WHERE u.id = ?"
        )
        .bind(other_user_id)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or_else(|_| format!("User {}", other_user_id))
    } else {
        data.name.clone().unwrap()
    };

    // Insert chat
    let insert_result = sqlx::query(
        "INSERT INTO chats (name, photo, description, chat_type, created_by, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(&chat_name)
    .bind(&data.name.as_ref().and_then(|_| None::<String>)) // photo
    .bind(&data.description)
    .bind(&data.chat_type)
    .bind(user.id)
    .execute(pool.get_ref())
    .await;

    let chat_id = match insert_result {
        Ok(result) => result.last_insert_id(),
        Err(e) => {
            log::error!("Failed to create chat: {}", e);
            return HttpResponse::InternalServerError().json("Failed to create chat");
        }
    };

    // Add creator as participant (admin for groups)
    let is_admin = data.chat_type == "group";
    if let Err(e) = sqlx::query(
        "INSERT INTO chat_participants (user_id, chat_id, is_admin, joined_at)
         VALUES (?, ?, ?, NOW())"
    )
    .bind(user.id)
    .bind(chat_id)
    .bind(is_admin)
    .execute(pool.get_ref())
    .await
    {
        log::error!("Failed to add creator as participant: {}", e);
        return HttpResponse::InternalServerError().json("Failed to add participants");
    }

    // Add other participants
    for &participant_id in &data.participant_ids {
        if let Err(e) = sqlx::query(
            "INSERT INTO chat_participants (user_id, chat_id, is_admin, joined_at)
             VALUES (?, ?, FALSE, NOW())"
        )
        .bind(participant_id)
        .bind(chat_id)
        .execute(pool.get_ref())
        .await
        {
            log::error!("Failed to add participant {}: {}", participant_id, e);
            // Continue adding other participants
        }
    }

    log::info!("Chat {} created by user {}", chat_id, user.id);
    HttpResponse::Created().json(serde_json::json!({ "chat_id": chat_id, "existed": false }))
}

/// GET /api/v1/chats/{chat_id}/messages - Get messages with pagination
#[get("/api/v1/chats/{chat_id}/messages")]
pub async fn get_chat_messages(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    chat_id: web::Path<u64>,
    query: web::Query<MessageQueryParams>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    // Verify user is participant
    match user.is_chat_participant(pool.get_ref(), *chat_id).await {
        Ok(true) => {},
        Ok(false) => return HttpResponse::Forbidden().json("You are not a participant of this chat"),
        Err(e) => {
            log::error!("Failed to verify chat participant: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    let limit = query.limit.unwrap_or(50).min(100); // Max 100 messages
    let offset = query.offset.unwrap_or(0);

    // Get messages with sender info
    let messages_query = r#"
        SELECT cm.id, cm.chat_id, cm.sender_id, cm.created_at, cm.updated_at,
               cm.type_message, cm.message, cm.file_path, cm.file_name, cm.file_size,
               cm.is_deleted, cm.reply_to_id
        FROM chat_messages cm
        WHERE cm.chat_id = ? AND cm.is_deleted = FALSE
        ORDER BY cm.created_at DESC
        LIMIT ? OFFSET ?
    "#;

    match sqlx::query_as::<_, ChatMessage>(messages_query)
        .bind(*chat_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(mut messages) => {
            // Reverse to get chronological order
            messages.reverse();
            HttpResponse::Ok().json(messages)
        },
        Err(e) => {
            log::error!("Failed to fetch messages: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch messages")
        }
    }
}

#[derive(serde::Deserialize)]
pub struct MessageQueryParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

/// POST /api/v1/chats/{chat_id}/messages - Send message (HTTP fallback to WebSocket)
#[post("/api/v1/chats/{chat_id}/messages")]
pub async fn send_message(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    chat_id: web::Path<u64>,
    data: web::Json<SendMessageRequest>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    // Verify user is participant
    match user.is_chat_participant(pool.get_ref(), *chat_id).await {
        Ok(true) => {},
        Ok(false) => return HttpResponse::Forbidden().json("You are not a participant of this chat"),
        Err(e) => {
            log::error!("Failed to verify chat participant: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    let type_msg = data.type_message.as_deref().unwrap_or("text");

    // Insert message
    let insert_result = sqlx::query(
        "INSERT INTO chat_messages (chat_id, sender_id, message, type_message, reply_to_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(*chat_id)
    .bind(user.id)
    .bind(&data.message)
    .bind(type_msg)
    .bind(data.reply_to_id)
    .execute(pool.get_ref())
    .await;

    match insert_result {
        Ok(result) => {
            let message_id = result.last_insert_id();
            HttpResponse::Created().json(serde_json::json!({ "message_id": message_id }))
        },
        Err(e) => {
            log::error!("Failed to send message: {}", e);
            HttpResponse::InternalServerError().json("Failed to send message")
        }
    }
}

/// POST /api/v1/chats/{chat_id}/participants - Add participants to group chat
#[post("/api/v1/chats/{chat_id}/participants")]
pub async fn add_participants(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    chat_id: web::Path<u64>,
    data: web::Json<AddParticipantsRequest>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    // Verify user is admin of the chat
    match user.is_chat_admin(pool.get_ref(), *chat_id).await {
        Ok(true) => {},
        Ok(false) => return HttpResponse::Forbidden().json("Only chat admins can add participants"),
        Err(e) => {
            log::error!("Failed to verify chat admin: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    let mut added = 0;
    for &user_id in &data.user_ids {
        // Check if already a participant
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE user_id = ? AND chat_id = ?)"
        )
        .bind(user_id)
        .bind(*chat_id)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(false);

        if exists {
            continue;
        }

        // Add participant
        if sqlx::query(
            "INSERT INTO chat_participants (user_id, chat_id, is_admin, joined_at)
             VALUES (?, ?, FALSE, NOW())"
        )
        .bind(user_id)
        .bind(*chat_id)
        .execute(pool.get_ref())
        .await
        .is_ok()
        {
            added += 1;
        }
    }

    HttpResponse::Ok().json(serde_json::json!({ "added": added }))
}

#[derive(serde::Deserialize)]
pub struct AddParticipantsRequest {
    user_ids: Vec<u64>,
}

/// DELETE /api/v1/chats/{chat_id}/participants/{user_id} - Remove participant from group chat
#[delete("/api/v1/chats/{chat_id}/participants/{user_id}")]
pub async fn remove_participant(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<(u64, u64)>,
) -> impl Responder {
    let (chat_id, target_user_id) = path.into_inner();

    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    // User can remove themselves, or admins can remove others
    let can_remove = if user.id == target_user_id {
        // Users can leave chats
        true
    } else {
        // Check if requester is admin
        match user.is_chat_admin(pool.get_ref(), chat_id).await {
            Ok(is_admin) => is_admin,
            Err(e) => {
                log::error!("Failed to verify chat admin: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        }
    };

    if !can_remove {
        return HttpResponse::Forbidden().json("You cannot remove this participant");
    }

    // Remove participant
    match sqlx::query("DELETE FROM chat_participants WHERE user_id = ? AND chat_id = ?")
        .bind(target_user_id)
        .bind(chat_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "removed": true })),
        Err(e) => {
            log::error!("Failed to remove participant: {}", e);
            HttpResponse::InternalServerError().json("Failed to remove participant")
        }
    }
}

/// GET /api/v1/chats/available-users - Get users available to chat with
#[get("/api/v1/chats/available-users")]
pub async fn get_available_users(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    match user.get_available_chat_users(pool.get_ref()).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            log::error!("Failed to fetch available users: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch users")
        }
    }
}

/// POST /api/v1/chats/{chat_id}/upload - Upload file in chat
#[post("/api/v1/chats/{chat_id}/upload")]
pub async fn upload_chat_file(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    chat_id: web::Path<u64>,
    multipart: Multipart,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    // Verify user is participant
    match user.is_chat_participant(pool.get_ref(), *chat_id).await {
        Ok(true) => {},
        Ok(false) => return HttpResponse::Forbidden().json("You are not a participant of this chat"),
        Err(e) => {
            log::error!("Failed to verify chat participant: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    // Parse multipart form data
    let hashmap = match parse_multipart(
        multipart,
        Some(&["pdf", "docx", "jpg", "jpeg", "png", "gif", "zip", "txt"]),
        Some(&[
            "application/pdf",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "image/jpeg",
            "image/png",
            "image/gif",
            "application/zip",
            "text/plain",
        ]),
        "./uploads/chat_files",
    )
    .await
    {
        Ok(h) => h,
        Err(e) => return HttpResponse::BadRequest().json(format!("Invalid upload: {}", e)),
    };

    let file_path = match hashmap.get("file") {
        Some(f) => String::from_utf8_lossy(f).to_string(),
        None => return HttpResponse::BadRequest().json("Missing file"),
    };

    // Extract filename and size from path
    let file_name = file_path.split('/').last().unwrap_or("file");

    // Determine message type based on file extension
    let msg_type = if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg")
                      || file_name.ends_with(".png") || file_name.ends_with(".gif") {
        "image"
    } else {
        "file"
    };

    // Insert message with file
    let insert_result = sqlx::query(
        "INSERT INTO chat_messages (chat_id, sender_id, message, type_message, file_path, file_name, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(*chat_id)
    .bind(user.id)
    .bind(file_name) // Message contains filename
    .bind(msg_type)
    .bind(&file_path)
    .bind(file_name)
    .execute(pool.get_ref())
    .await;

    match insert_result {
        Ok(result) => {
            let message_id = result.last_insert_id();
            HttpResponse::Created().json(serde_json::json!({
                "message_id": message_id,
                "file_path": file_path
            }))
        },
        Err(e) => {
            log::error!("Failed to upload file: {}", e);
            HttpResponse::InternalServerError().json("Failed to upload file")
        }
    }
}

/// PUT /api/v1/chats/{chat_id}/read - Mark all messages in chat as read
#[put("/api/v1/chats/{chat_id}/read")]
pub async fn mark_chat_as_read(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    chat_id: web::Path<u64>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    // Verify user is participant
    match user.is_chat_participant(pool.get_ref(), *chat_id).await {
        Ok(true) => {},
        Ok(false) => return HttpResponse::Forbidden().json("You are not a participant of this chat"),
        Err(e) => {
            log::error!("Failed to verify chat participant: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    // Mark all messages as read
    let result = sqlx::query(
        r#"
        INSERT INTO reads (message_id, reader_id, read_at)
        SELECT cm.id, ?, NOW()
        FROM chat_messages cm
        WHERE cm.chat_id = ? AND cm.sender_id != ?
        ON DUPLICATE KEY UPDATE read_at = NOW()
        "#
    )
    .bind(user.id)
    .bind(*chat_id)
    .bind(user.id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "marked_read": true })),
        Err(e) => {
            log::error!("Failed to mark messages as read: {}", e);
            HttpResponse::InternalServerError().json("Failed to mark as read")
        }
    }
}
