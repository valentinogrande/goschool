use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::filters::AssistanceFilter;
use crate::structs::{NewAssistance, UpdateAssistance};
use crate::jwt::validate;
use crate::traits::{Get, Post, Update, Delete};

#[get("/api/v1/chat_messages/")]
pub async fn get_chat_messages(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<AssistanceFilter>,
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
    
    let assistance = match user.get_assistance(&pool, filter.into_inner()).await {
        Ok(m) => m,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(assistance)
}

#[post("/api/v1/chat_messages/")]
pub async fn post_chat_messages(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    assistance: web::Json<NewAssistance>,
) -> impl Responder {
    let jwt = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(jwt.value()) {
        Ok(t) => t,
        Err(e) => return HttpResponse::Unauthorized().body(e.to_string()),
    };

    let user = token.claims.user;

    user.post_assistance(&pool, assistance.into_inner()).await
}

#[put("/api/v1/chat_messages/{id}")]
pub async fn update_chat_messages(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
    data: web::Json<UpdateAssistance>,
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
    user.update_assistance(pool.get_ref(), *id, data.into_inner()).await
}

#[delete("/api/v1/chat_messages/{id}")]
pub async fn delete_chat_messages(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
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
    user.delete_assistance(pool.get_ref(), *id).await
}
