use actix_web::{get, HttpRequest, HttpResponse, Responder, web};
use sqlx::MySqlPool;

use crate::jwt::validate;
use crate::filters::UserFilter;
use crate::traits::Get;

#[get("/api/v1/students/")]
pub async fn get_students(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    filter: web::Query<UserFilter>
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
    let users_id = user.get_students(pool, Some(filter.into_inner()));
    if let Ok(i) = users_id.await {   
        HttpResponse::Ok().json(i)   
    }
    else {
        HttpResponse::InternalServerError().finish()
    }
}
