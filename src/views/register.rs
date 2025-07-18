use actix_web::{post, web, HttpRequest, HttpResponse, Responder, get};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};
use std::env;

use crate::{jwt::validate, structs::NewUser, structs::Role};

#[post("/api/v1/register/")]
pub async fn register(
    pool: web::Data<MySqlPool>,
    user: web::Json<NewUser>,
    req: HttpRequest,
) -> impl Responder {

    let hashed_pass = match hash(&user.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };


    log::info!("Registering user: {}, pass {}", user.email, user.password);
    
    let debug = env::var("DEBUG").unwrap();

    if debug != "true" {
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
    }

    let _query = match sqlx::query("INSERT INTO users (password, email) VALUES (?, ?)")
            .bind(&hashed_pass)
            .bind(&user.email)
        .execute(pool.get_ref())
        .await {
        Ok(g) => g,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let _query = match sqlx::query("INSERT INTO roles (user_id, role) VALUES (?, ?)")
            .bind(_query.last_insert_id())
            .bind(&user.role)
        .execute(pool.get_ref())
        .await {
        Ok(g) => g,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Created().finish()
}

#[get("/api/v1/register_testing_users/")]
pub async fn register_testing_users(req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder{

    let users: Vec<String> = vec![
        "admin".to_string(),
        "student".to_string(),
        "preceptor".to_string(),
        "father".to_string(),
        "teacher".to_string(),
    ];

    let debug = env::var("DEBUG").unwrap();

    if debug != "true" {
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
    }

    for user in users.iter() {
        let hash = hash(user, DEFAULT_COST).unwrap();
        let res = match sqlx::query("INSERT INTO users (password, email) VALUES (?,?)")
    .bind(&hash)
    .bind(user)
    .execute(pool.get_ref()).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let user_id = res.last_insert_id();

    let _res = match sqlx::query("INSERT INTO roles (user_id, role) VALUES (?,?)")
        .bind(user_id)
        .bind(user)
        .execute(pool.get_ref()).await {
            Ok(_) => {},
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
        };
    }

    let email = "valentinogrande972@gmail.com";

    let pass = "student";

    let hash = hash(pass, DEFAULT_COST).unwrap();

    let res = match sqlx::query("INSERT INTO users (password, email) VALUES (?,?)")
    .bind(&hash)
    .bind(email)
    .execute(pool.get_ref()).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let user_id = res.last_insert_id();

    let _res = match sqlx::query("INSERT INTO roles (user_id, role) VALUES (?,?)")
        .bind(user_id)
        .bind("student")
        .execute(pool.get_ref()).await {
            Ok(_) => {},
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    
    let _res = match sqlx::query(
        "INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(6)
    .bind("valentino grande")
    .bind("2024-07-18 15:30:00")
    .bind("santa coloma 9282")
    .bind("543412115831")
    .execute(pool.get_ref())
    .await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };


    HttpResponse::Created().finish()
}
