use actix_web::cookie::Cookie;
use actix_web::{post, web, HttpResponse,  Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::verify;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use std::fs;

use crate::Claims;
use crate::user::MySelf;
use crate::structs::CredentialsRole;

#[post("/api/v1/login/")]
pub async fn login(
    pool: web::Data<MySqlPool>,
    creds: web::Json<CredentialsRole>,
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
    
    let role_existance: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM roles WHERE user_id = ? AND role = ?)")
        .bind(user_id)
        .bind(&creds.role)
        .fetch_one(pool.get_ref())
        .await {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if !role_existance {
        return HttpResponse::Unauthorized().finish();
    }


    
    let claims = Claims::new(MySelf::new(user_id as u64, creds.role.clone()));

    let private_key_pem = match fs::read("ecc_private_key.pem") {
        Ok(k) => k,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let encoding_key = match EncodingKey::from_ec_pem(&private_key_pem) {
        Ok(k) => k,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let token = match encode(
        &Header::new(Algorithm::ES256),
        &claims,
        &encoding_key,
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
