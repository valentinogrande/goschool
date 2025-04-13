use actix_web::cookie::Cookie;
use actix_web::{post, web, HttpResponse,  Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::verify;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use crate::Credentials;
use crate::Claims;

#[post("/api/v1/login/")]
pub async fn login(
    pool: web::Data<MySqlPool>,
    creds: web::Json<Credentials>,
) -> impl Responder {
    let result: (u64,String) = match sqlx::query_as("SELECT id, password FROM users WHERE email = ?")
        .bind(&creds.email)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(record) => record,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid credentials"),
    };

    let hashed_pass = result.1;
    let valid = verify(&creds.password, &hashed_pass).unwrap_or(false);

    if !valid {
        return HttpResponse::Unauthorized().json("Invalid credentials");
    }

    let user_id = result.0;
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
