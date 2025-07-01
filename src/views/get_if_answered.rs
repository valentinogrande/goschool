use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post};
use sqlx::mysql::MySqlPool;
use serde::Deserialize;

use crate::jwt::validate;
use crate::traits::Post;

#[derive(Deserialize)]
pub struct HomeworkId {
    pub homework_id: u64,
}

#[derive(Deserialize)]
pub struct SelfassessableId {
    pub selfassessable_id: u64,
}

#[post("/api/v1/get_if_homework_answered/")]
pub async fn get_if_homework_answered(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    homework_id: web::Json<HomeworkId>,
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

    let is_answered = match user.get_is_homework_answered(&pool, homework_id.homework_id).await { 
        Ok(g) => g,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(is_answered)
}

#[get("/api/v1/get_if_selfassessable_answered/")]
pub async fn get_if_selfassessable_answered(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    selfassesssable_id: web::Json<SelfassessableId>,
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

    let is_answered = match user.get_is_selfassessable_answered(&pool, selfassesssable_id.selfassessable_id).await { 
        Ok(g) => g,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(is_answered)
}
