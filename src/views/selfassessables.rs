use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::structs::NewSubmissionSelfAssessable;
use crate::traits::{Post, Get};
use crate::filters::SelfassessableFilter;

#[get("/api/v1/selfassessables/")]
pub async fn get_selfassessables(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<SelfassessableFilter>,
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
    
    let selfassessables = match user.get_selfassessables(&pool, filter.into_inner()).await {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };
    
    HttpResponse::Ok().json(selfassessables)
}


#[post("/api/v1/selfassessables/")]
pub async fn post_selfassessable_submission(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    task_submission: web::Json<NewSubmissionSelfAssessable>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    
    let user = token.claims.user;
        
    user.post_submission_selfassessable(&pool, task_submission.into_inner()).await
}

#[get("/api/v1/selfassessables_responses/")]
pub async fn get_selfassessables_responses(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<SelfassessableFilter>,
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
    
    let selfassessables = match user.get_selfassessables_responses(&pool, filter.into_inner()).await {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };
    
    HttpResponse::Ok().json(selfassessables)
    
}


