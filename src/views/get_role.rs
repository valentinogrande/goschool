use actix_web::{get, HttpRequest, HttpResponse, Responder};

use crate::jwt::validate;


#[get("/api/v1/get_role/")]
pub async fn get_role(
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

    let role = token.claims.role;

    HttpResponse::Ok().json(role)   
}
