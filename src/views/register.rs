use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};

use crate::{jwt::validate, user::NewUser, user::Role};

#[post("/api/v1/register/")]
pub async fn register(
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
    let token = match validate(jwt.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    
    let role = token.claims.role;
    
    if role != Role::admin {
        return HttpResponse::Unauthorized().finish();
    }

    let _query = match sqlx::query("INSERT INTO users (password, email, role) VALUES (?, ?, ?)")
            .bind(&hashed_pass)
            .bind(&user.email)
            .bind(&user.role)
        .execute(pool.get_ref())
        .await {
        Ok(g) => g,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    HttpResponse::Created().finish()
}
