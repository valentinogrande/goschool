use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use actix_multipart::Multipart;

use crate::jwt::validate;
use crate::traits::Post;


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
