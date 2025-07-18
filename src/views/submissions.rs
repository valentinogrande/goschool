use actix_web::{post, web, HttpRequest, HttpResponse, Responder, put, delete};
use sqlx::mysql::MySqlPool;
use actix_multipart::Multipart;

use crate::jwt::validate;
use crate::traits::Post;
use crate::structs::UpdateSubmission;
use crate::traits::{Update, Delete};


#[post("/api/v1/homework_submission/")]
pub async fn post_homework_submission(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
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

    user.post_submission(&pool, multipart).await
}

#[put("/api/v1/homework_submission/{id}")]
pub async fn update_submission(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
    data: web::Json<UpdateSubmission>,
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
    user.update_submission(pool.get_ref(), *id, data.into_inner()).await
}

#[delete("/api/v1/homework_submission/{id}")]
pub async fn delete_submission(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
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
    user.delete_submission(pool.get_ref(), *id).await
}
