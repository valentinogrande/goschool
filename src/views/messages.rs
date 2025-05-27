use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post};
use sqlx::mysql::MySqlPool;

use crate::filters::MessageFilter;
use crate::structs::NewMessage;
use crate::jwt::validate;
use crate::traits::{Get, Post};

#[get("/api/v1/messages/")]
pub async fn get_messages(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<MessageFilter>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };

    let user = token.claims.user;
    
    let messages = match user.get_messages(&pool, Some(filter.into_inner())).await {
        Ok(m) => m,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(messages)
}

#[post("/api/v1/messages/")]
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

    let user = token.claims.user;

    user.post_message(&pool, message.into_inner()).await
}
