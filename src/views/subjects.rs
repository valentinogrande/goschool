use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use sqlx::mysql::MySqlPool;

use crate::filters::SubjectFilter;
use crate::jwt::validate;
use crate::traits::Get;

#[get("/api/v1/subjects/")]
pub async fn get_subjects(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<SubjectFilter>,
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
    
    let subjects = match user.get_subjects(&pool, filter.into_inner()).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(subjects)
}
