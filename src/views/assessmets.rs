use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post, put, delete};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::traits::{Get, Post, Update, Delete};
use crate::structs::Payload;
use crate::structs::{UpdateAssessment};
use crate::filters::{AssessmentFilter, SubjectFilter, UserFilter};

#[get("/api/v1/assessments/")]
pub async fn get_assessments(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<AssessmentFilter>,
    subject_filter: web::Query<SubjectFilter>,
    person_filter: web::Query<UserFilter>,
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
    let assessments = match user.get_assessments(&pool, filter.into_inner(), subject_filter.into_inner(), person_filter.into_inner()).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(assessments)
}

#[post("/api/v1/assessments/")]
pub async fn post_assessment(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<Payload>,
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
    
    user.post_assessment(&pool, payload.into_inner()).await
}

#[put("/api/v1/assessments/{id}")]
pub async fn update_assessment(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
    data: web::Json<UpdateAssessment>,
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
    user.update_assessment(pool.get_ref(), *id, data.into_inner()).await
}

#[delete("/api/v1/assessments/{id}")]
pub async fn delete_assessment(
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
    user.delete_assessment(pool.get_ref(), *id).await
}