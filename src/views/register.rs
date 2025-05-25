use actix_web::{post, web, HttpRequest, HttpResponse, Responder, get};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};

use crate::{jwt::validate, user::NewUser, structs::Role};

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
    
    let role = token.claims.user.role;
    
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

#[get("/api/v1/register_testing_users/")]
pub async fn register_testing_users(pool: web::Data<MySqlPool>) -> impl Responder{
    let users: Vec<&str> = Vec::from(["admin","student","preceptor","father","teacher"]);
    for (i, user) in users.iter().enumerate() {
        let hash = hash(user, DEFAULT_COST).unwrap();
        let _res = match sqlx::query("INSERT INTO users (password, email) VALUES (?,?)")
        .bind(&hash)
        .bind(user)
        .execute(pool.get_ref()).await{
            Ok(_) => {},
            Err(_) => {}
        };

        let i = i +1;
        log::info!("user: {:?}, id : {}", user,i);
        let _res = match sqlx::query("INSERT INTO roles (user_id, role) VALUES (?,?)")
        .bind(i as i32)
        .bind(user)
        .execute(pool.get_ref()).await{
            Ok(r) => r,
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
        };
    }
    HttpResponse::Created().finish()
}
