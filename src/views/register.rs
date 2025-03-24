use actix_web::{post, web, HttpResponse,  Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};

use crate::NewUser;


#[utoipa::path(
    post,
    path = "/api/v1/register/",
    request_body(content = NewUser, description = "User registration data", content_type = "application/json"),
    responses(
        (status = 201, description = "User created successfully"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/api/v1/register/")]
pub async fn create_user(pool: web::Data<MySqlPool>, user: web::Json<NewUser>) -> impl Responder {
    let hashed_pass = hash(&user.password, DEFAULT_COST);
    if let Err(e) = hashed_pass {
        return HttpResponse::InternalServerError().json(e.to_string())
    }
    else {
        let hashed_pass = hashed_pass.unwrap();
        let result = sqlx::query("INSERT INTO user (FullName, password, email) VALUES (?,?,?)")
            .bind(user.username.clone())
            .bind(hashed_pass)
            .bind(user.email.clone())
            .execute(pool.get_ref())
            .await;

        match result {
            Ok(_) => HttpResponse::Created().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}
