use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::traits::Get;
use crate::filters::SelfassessableFilter;


#[get("/api/v1/selfassessables_pending_grades/")]
pub async fn get_selfassessables_pending_grades(
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
    
    let selfassessables = match user.get_pending_selfassessables_grades(&pool, filter.into_inner()).await {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };
    
    HttpResponse::Ok().json(selfassessables)
}
