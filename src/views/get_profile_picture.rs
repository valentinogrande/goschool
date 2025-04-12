use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use serde::Serialize;
use std::env;

use crate::jwt::validate;

#[derive(Serialize)]
struct PhotoUrlResponse {
    url: String,
}

#[get("/api/v1/get_profile_picture/")]
pub async fn get_profile_picture(req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder {
    
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.subject;
    let photo_filename: String = match sqlx::query_scalar("SELECT photo FROM users WHERE id = ?")
        .bind(user_id as i64)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(path)) => path,
        Ok(None) => "default.jpg".to_string(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let base_url = env::var("BASE_URL").expect("BASE_URL must be set");
    let url = format!("{}/uploads/profile_pictures/{}", base_url, photo_filename);

    HttpResponse::Ok().json(PhotoUrlResponse { url })
}

