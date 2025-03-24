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
        (status = 401, description = "Invalid credentials")
    )
)]
#[post("/api/v1/login/")]
pub async fn login(pool: web::Data<MySqlPool>, creds: web::Json<Credentials>) -> impl Responder {
    let password_from_db = sqlx::query("SELECT userid,password FROM user WHERE email = ?")
        .bind(creds.email.clone())
        .fetch_one(pool.get_ref())
        .await;

    if let Ok(record) = password_from_db {
        let password = record.get::<String, &str>("password");
        if verify(&creds.password, &password).unwrap_or(false) {
            let claims = Claims::new(record.get::<i32, &str>("userid") as usize);
            let secret = "prod_secret";
            let token = encode(
                &Header::new(Algorithm::HS256),
                &claims,
                &EncodingKey::from_secret(secret.as_ref()),
            );

            HttpResponse::Ok().json(token.unwrap())
        } else {
            HttpResponse::Unauthorized().json("Invalid credentials")
        }
    } else {
        HttpResponse::Unauthorized().json("Invalid credentials")
    }
}
