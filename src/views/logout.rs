use actix_web::{post, HttpResponse, Responder};
use actix_web::cookie::{Cookie, time::Duration};

#[post("/api/v1/logout/")]
pub async fn logout() -> impl Responder {
    let expired_cookie = Cookie::build("jwt", "")
        .path("/")
        .http_only(true)
        .secure(false)
        .max_age(Duration::seconds(0))
        .finish();

    HttpResponse::Ok()
        .cookie(expired_cookie)
        .json("logout success")
}
