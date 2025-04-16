use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::user::Role;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewMessage {
    courses: String,
    title: String,
    message: String,
}


#[post("/api/v1/post_message/")]
pub async fn post_message(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    message: web::Json<NewMessage>,
) -> impl Responder {
    let jwt = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(jwt.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.subject as u64;

    let role = token.claims.role;


    let courses: Vec<u64> = message
    .courses
    .split(',')
    .filter_map(|s| s.trim().parse::<u64>().ok())
    .collect();
    // check if courses are valid
    for course in courses.iter() {
        if *course <= 0 || *course > 36 {
            return HttpResponse::BadRequest().finish();
        } 
    }

    if role == Role::preceptor {
        let preceptor_courses: Vec<u64> = match sqlx::query_scalar::<_, u64>("SELECT id FROM courses WHERE preceptor_id = ?")
        .bind(user_id)
        .fetch_all(pool.get_ref())
    .await {
            Ok(r) => r,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

        if !courses.iter().all(|&course| preceptor_courses.contains(&course)) {
            return HttpResponse::Unauthorized().json("Not authorized to post message for these courses");
        }
    
    }
    else if role == Role::admin{}
    else {
        return HttpResponse::BadRequest().finish();
    }

    
    let insert_result = sqlx::query("INSERT INTO messages (message, sender_id, title) VALUES (?, ?, ?)")
        .bind(&message.message)
        .bind(user_id)
        .bind(&message.title)
        .execute(pool.get_ref())
        .await;

    let message_id = match insert_result {
        Ok(ref result) => result.last_insert_id(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    for course in courses.iter() {
        let _insert_result = match sqlx::query("INSERT INTO message_courses (course_id, message_id) VALUES (?,?)")
        .bind(course)
        .bind(message_id)
        .execute(pool.get_ref())
        .await {
            Ok(r) =>  r,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };
    }

    match insert_result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}
