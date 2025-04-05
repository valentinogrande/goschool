use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};

use crate::{jwt::validate, user::NewUser};


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
pub async fn create_user(
    pool: web::Data<MySqlPool>,
    user: web::Json<NewUser>,
    req: HttpRequest,
) -> impl Responder {
    let hashed_pass = match hash(&user.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let is_admin = req
        .cookie("jwt")
        .and_then(|cookie| validate(cookie.value().to_string()).ok())
        .map_or(false, |token| token.claims.subject == 2);

    let query = if is_admin {
        sqlx::query("INSERT INTO users (password, email, is_teacher) VALUES (?, ?, ?)")
            .bind(&hashed_pass)
            .bind(&user.email)
            .bind(user.is_teacher)
    } else {
        sqlx::query("INSERT INTO users (password, email) VALUES (?, ?)")
            .bind(&hashed_pass)
            .bind(&user.email)
    };

    match query.execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
