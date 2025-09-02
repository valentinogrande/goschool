use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post, put};
use sqlx::mysql::MySqlPool;

use crate::filters::MessageFilter;
use crate::structs::NewMessage;
use crate::structs::UpdateMessage;
use crate::jwt::validate;
use crate::traits::{Get, Post, Update};

#[get("/api/v1/assistance/")]
pub async fn get_assisstance(
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
    
    let messages = match user.get_messages(&pool, filter.into_inner()).await {
        Ok(m) => m,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(messages)
}

#[post("/api/v1/assistance/")]
pub async fn post_assistance(
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
        Err(e) => return HttpResponse::Unauthorized().body(e.to_string()),
    };

    let user = token.claims.user;

    user.post_message(&pool, message.into_inner()).await
}

#[put("/api/v1/assistance/{id}")]
pub async fn update_assistance(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
    data: web::Json<UpdateMessage>,
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
    user.update_message(pool.get_ref(), *id, data.into_inner()).await
}
