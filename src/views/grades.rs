use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post, put, delete};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::structs::NewGrade;
use crate::filters::GradeFilter;
use crate::traits::{Get, Post, Update, Delete};
use crate::structs::UpdateGrade;


#[get("/api/v1/grades/")]
pub async fn get_grades(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<GradeFilter>,
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

    let grades = match user.get_grades(&pool, filter.into_inner()).await { 
        Ok(g) => g,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(grades)
}

#[post("/api/v1/grades/")]
pub async fn post_grade(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    grade: web::Json<NewGrade>,
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
    
    user.post_grade(&pool, grade.into_inner()).await
}

#[put("/api/v1/grades/{id}")]
pub async fn update_grade(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
    data: web::Json<UpdateGrade>,
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
    user.update_grade(pool.get_ref(), *id, data.into_inner()).await
}

#[delete("/api/v1/grades/{id}")]
pub async fn delete_grade(
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
    user.delete_grade(pool.get_ref(), *id).await
}
