use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::filters::MessageFilter;
use crate::jwt::validate;

#[get("/api/v1/get_messages/")]
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
