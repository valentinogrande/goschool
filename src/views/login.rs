use actix_web::cookie::Cookie;
use actix_web::{post, web, HttpResponse,  Responder};
use sqlx::mysql::MySqlPool;
use sqlx::Row;
use bcrypt::verify;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use crate::Credentials;
use crate::Claims;


#[utoipa::path(
    post,
    path = "/api/v1/login/",
    request_body(content = Credentials, description = "User credentials", content_type = "application/json"),
    responses(
        (status = 200, description = "Login successful", body = String),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Json parsing error"),
    )
)]

#[post("/api/v1/login/")]
pub async fn login(
    pool: web::Data<MySqlPool>,
    creds: web::Json<Credentials>,
) -> impl Responder {
    let row = match sqlx::query("SELECT id, password FROM users WHERE email = ?")
        .bind(&creds.email)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(record) => record,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid credentials"),
    };

    let stored_pass = row.get::<String, &str>("password");
    let valid = verify(&creds.password, &stored_pass).unwrap_or(false);

    if !valid {
        return HttpResponse::Unauthorized().json("Invalid credentials");
    }

    let user_id = row.get::<i32, &str>("id");
    let claims = Claims::new(user_id as usize);
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECTRET should be setted");
    let token = match encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let _ = match sqlx::query("UPDATE users SET last_login = NOW() WHERE id = ?")
        .bind(user_id)
        .execute(pool.get_ref())
        .await{
        Ok(_) => 0,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let cookie = Cookie::build("jwt", token)
        .path("/")
        .http_only(true)
        .secure(false)
        .finish();

    HttpResponse::Ok().cookie(cookie).json("login success")
}
