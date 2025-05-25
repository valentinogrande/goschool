use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::filters::SubjectFilter;


#[get("/api/v1/subjetcs/")]
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
    let subjects = match user.get_subjects(&pool, Some(filter.into_inner())).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(subjects)
}
