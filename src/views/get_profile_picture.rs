use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::structs::PhotoUrlResponse;

#[get("/api/v1/get_profile_picture/")]
pub async fn get_profile_picture(req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder {
    
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    
    let url = match token.claims.user.get_profile_picture(&pool).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };
    
    HttpResponse::Ok().json(PhotoUrlResponse { url })
}
