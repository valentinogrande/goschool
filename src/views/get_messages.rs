use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::QueryBuilder;

use crate::jwt::validate;
use crate::user::Role;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub sender_id: u64,
    pub created_at: Option<DateTime<Utc>>,
}

#[get("/api/v1/get_messages/{student_id}/")]
pub async fn get_messages(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    student_id: web::Path<u64>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };

    let user_id = token.claims.subject as u64;
    let mut student_id = student_id.into_inner();
    
    if student_id == 0{
       student_id = user_id; 
    }

    let role = token.claims.role;
    
    let messages_ids: Vec<u64>;    

    if role == Role::father {
        let students_id: Vec<u64> = match sqlx::query_scalar("SELECT student_id FROM families WHERE father_id = ?")
            .bind(user_id)
            .fetch_all(pool.get_ref())
            .await
        {
            Ok(r) => r,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

        if !students_id.contains(&student_id) {
            return HttpResponse::Unauthorized().json("Not authorized to access this student's data");
        }
        let ids: Vec<u64> = match sqlx::query_scalar::<_,u64>("SELECT mc.message_id FROM message_courses mc JOIN users u ON mc.course_id = u.course_id WHERE u.id = ?")
            .bind(student_id)
            .fetch_all(pool.get_ref())
            .await {
            Ok(r) => r,
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
        };
        messages_ids = ids;
    }
    else if role == Role::preceptor{
        let ids: Vec<u64> = match sqlx::query_scalar::<_,u64>("SELECT mc.message_id FROM message_courses mc JOIN courses c ON mc.course_id = c.id WHERE c.preceptor_id = ?")
            .bind(user_id)
            .fetch_all(pool.get_ref())
            .await {
            Ok(r) => r,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };
        messages_ids = ids;
    }
    else if role == Role::student {
        if user_id != student_id {
            return HttpResponse::Unauthorized().finish();
        }

        let ids: Vec<u64> = match sqlx::query_scalar::<_,u64>("SELECT mc.message_id FROM message_courses mc JOIN users u ON mc.course_id = u.course_id WHERE u.id = ?")
            .bind(user_id)
            .fetch_all(pool.get_ref())
            .await {
            Ok(r) => r,
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
        };
        messages_ids = ids;
    }
    else if role == Role::admin{
        let ids: Vec<u64> = match sqlx::query_scalar::<_,u64>("SELECT message_id FROM message_courses")
            .fetch_all(pool.get_ref())
            .await {
            Ok(r) => r,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };
        messages_ids = ids;
    }
    else {
        return HttpResponse::Unauthorized().finish();
    }

    let mut query = QueryBuilder::new("SELECT * FROM messages WHERE id IN (");

    for (index, id) in messages_ids.iter().enumerate() {
        if index > 0 {
            query.push(", ");
        }
        query.push_bind(id);
    }

    query.push(")");

    let messages: Vec<Message> = match query
        .build_query_as()
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(messages)
}
