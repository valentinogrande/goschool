use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, put, delete};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::traits::{Delete, Get, Post, Update};
use crate::filters::TimetableFilter;
use crate::structs::{NewTimetable, UpdateTimetable};

#[get("/api/v1/timetables/")]
pub async fn get_timetable(
    pool: web::Data<MySqlPool>,
    filter: web::Query<TimetableFilter>,
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
    
    let timetables = match user.get_timetables(&pool, filter.into_inner()).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(timetables)
}

#[post("/api/v1/timetables/")]
pub async fn post_timetable(
    pool: web::Data<MySqlPool>,
    timetable: web::Json<NewTimetable>,
    req: HttpRequest
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie")
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT cookie")
    };
    
    let user = token.claims.user;

    user.post_timetable(&pool, timetable.into_inner()).await
}

#[put("/api/v1/timetable/{id}")]
pub async fn update_timetable(
    pool: web::Data<MySqlPool>,
    timetable: web::Json<UpdateTimetable>,
    timetable_id: web::Path<u64>,
    req: HttpRequest
) -> impl Responder {

    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie")
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT cookie")
    };

    let user = token.claims.user;

    user.update_timetable(&pool, timetable_id.into_inner(), timetable.into_inner()).await
}

#[delete("/api/v1/timetable/{id}")]
pub async fn delete_timetable(
    pool: web::Data<MySqlPool>,
    timetable_id: web::Path<u64>,
    req: HttpRequest
) -> impl Responder {

    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie")
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT cookie")
    };

    let user = token.claims.user;

    user.delete_timetable(&pool, timetable_id.into_inner()).await
}
