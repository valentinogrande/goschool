use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::traits::Get;
use crate::filters::TimetableFilter;


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
