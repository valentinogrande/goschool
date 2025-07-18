use actix_web::{get, web, HttpRequest, HttpResponse, Responder, put, delete};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::traits::{Get, Update, Delete};
use crate::structs::UpdatePersonalData;

#[get("/api/v1/personal_data/")]
pub async fn get_personal_data(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
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
    
    let personal_data = match user.get_personal_data(&pool).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };
    HttpResponse::Ok().json(personal_data)
}

#[get("/api/v1/public_personal_data/")]
pub async fn get_public_personal_data(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<crate::filters::UserFilter>,
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
    
    let personal_data = match user.get_public_personal_data(&pool, filter.into_inner()).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(personal_data)
}

#[put("/api/v1/personal_data/{user_id}")]
pub async fn update_personal_data(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    user_id: web::Path<u64>,
    data: web::Json<UpdatePersonalData>,
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
    user.update_personal_data(pool.get_ref(), *user_id, data.into_inner()).await
}

#[delete("/api/v1/personal_data/{user_id}")]
pub async fn delete_personal_data(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    user_id: web::Path<u64>,
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
    user.delete_personal_data(pool.get_ref(), *user_id).await
}
