use actix_web::{get, HttpRequest, HttpResponse, Responder};

use crate::jwt::validate;

#[get("/api/v1/verify_token/")]
pub async fn verify_token(
    req: HttpRequest,
) -> impl Responder {
    
    let jwt = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let _token = match validate(jwt.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    HttpResponse::Ok().json("json web token is valid")
}
