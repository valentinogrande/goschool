use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post, put, delete};
use sqlx::mysql::MySqlPool;
use actix_multipart::Multipart;

use crate::jwt::validate;
use crate::structs::PhotoUrlResponse;
use crate::traits::{Get, Post, Update, Delete};

#[get("/api/v1/profile_pictures/")]
pub async fn get_profile_picture(req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder {
    
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    
    let url = match token.claims.user.get_profile_picture(&pool).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };
    
    HttpResponse::Ok().json(PhotoUrlResponse { url })
}

#[post("/api/v1/profile_pictures/")]
pub async fn post_profile_picture(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    task_submission: Multipart,
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

    user.post_profile_picture(&pool, task_submission).await
}

#[put("/api/v1/profile_pictures/{user_id}")]
pub async fn update_profile_picture(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    user_id: web::Path<u64>,
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
    user.update_profile_picture(pool.get_ref(), *user_id, multipart).await
}

#[delete("/api/v1/profile_pictures/{user_id}")]
pub async fn delete_profile_picture(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    user_id: web::Path<u64>,
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
    user.delete_profile_picture(pool.get_ref(), *user_id).await
}
