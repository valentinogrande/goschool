use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post, put, delete};
use sqlx::mysql::MySqlPool;

use crate::filters::DisciplinarySanctionFilter;
use crate::structs::NewDisciplinarySanction;
use crate::structs::UpdateDisciplinarySanction;
use crate::jwt::validate;
use crate::traits::{Get, Post, Update, Delete};

#[get("/api/v1/disciplinary_sanction/")]
pub async fn get_disciplinary_sanction(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<DisciplinarySanctionFilter>,
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
    
    let disciplinary_sanctions = match user.get_disciplinary_sanction(&pool, filter.into_inner()).await {
        Ok(m) => m,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(disciplinary_sanctions)
}

#[post("/api/v1/disciplinary_sanction/")]
pub async fn post_disciplinary_sanction(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    disciplinary_sanctions: web::Json<NewDisciplinarySanction>,
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

    user.post_disciplinary_sanction(&pool, disciplinary_sanctions.into_inner()).await
}

#[put("/api/v1/disciplinary_sanction/{id}")]
pub async fn update_disciplinary_sanction(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
    data: web::Json<UpdateDisciplinarySanction>,
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
    user.update_disciplinary_sanction(pool.get_ref(), *id, data.into_inner()).await
}

#[delete("/api/v1/disciplinary_sanction/{id}")]
pub async fn delete_disciplinary_sanction(
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
    user.delete_disciplinary_sanction(pool.get_ref(), *id).await
}
