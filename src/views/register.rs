use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};

use crate::{jwt::validate, user::NewUser};

#[post("/api/v1/register/")]
pub async fn create_user(
    pool: web::Data<MySqlPool>,
    user: web::Json<NewUser>,
    req: HttpRequest,
) -> impl Responder {
    let hashed_pass = match hash(&user.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let jwt = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let token = match validate(jwt.value().to_string()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    let is_admin = match sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE id = ?")
        .bind(token.claims.subject as i32)
        .fetch_one(pool.get_ref())
        .await{
        Ok(r) => r == "admin",
        Err(_) => return HttpResponse:: InternalServerError().body("role is invalid"),
    };
    
    let query = if is_admin {
        sqlx::query("INSERT INTO users (password, email, role) VALUES (?, ?, ?)")
            .bind(&hashed_pass)
            .bind(&user.email)
            .bind(&user.role)
    } else {
        return HttpResponse::Unauthorized().finish();
    };

    match query.execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
