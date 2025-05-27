use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::traits::{Get, Post};
use crate::structs::Payload;
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
    let assessments = match user.get_assessments(&pool, Some(filter.into_inner()), Some(subject_filter.into_inner()), Some(person_filter.into_inner())).await {
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
