use actix_web::{post, web, HttpRequest, HttpResponse, Responder, get};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};
use std::env;
use tokio::time::{timeout, Duration};

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
pub async fn register_testing_users(pool: web::Data<MySqlPool>) -> impl Responder {
    let users = vec![
        ("admin", "admin"),
        ("student", "student"),
        ("preceptor", "preceptor"),
        ("father", "father"),
        ("teacher", "teacher"),
        ("valentinogrande972@gmail.com", "student"),
    ];

    //let debug = env::var("DEBUG").unwrap_or_default();
    //if debug != "true" {
    //    let jwt = match req.cookie("jwt") {
    //        Some(c) => c,
    //        None => return HttpResponse::Unauthorized().finish(),
    //    };
    //    let token = match validate(jwt.value()) {
    //        Ok(t) => t,
    //        Err(_) => return HttpResponse::Unauthorized().finish(),
    //    };
    //    if token.claims.user.role != Role::admin {
    //        return HttpResponse::Unauthorized().finish();
    //    }
    //}
    let mut i = 0;
    for (email, password) in users {
        i=i+1;
        let hash = match hash(password, DEFAULT_COST) {
            Ok(h) => h,
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
        };

        let user_id = match timeout(Duration::from_secs(5), async {
            sqlx::query("INSERT INTO users (password, email) VALUES (?, ?)")
                .bind(&hash)
                .bind(email)
                .execute(pool.get_ref())
                .await
        })
        .await
        {
            Ok(Ok(res)) => res.last_insert_id(),
            Ok(Err(e)) => return HttpResponse::InternalServerError().body(e.to_string()),
            Err(_) => return HttpResponse::RequestTimeout().body("Insert en users demoró demasiado."),
        };

        let _insert_role = match timeout(Duration::from_secs(5), async {
            sqlx::query("INSERT INTO roles (user_id, role) VALUES (?, ?)")
                .bind(user_id)
                .bind(password)
                .execute(pool.get_ref())
                .await
        })
        .await
        {
            Ok(Ok(_)) => (),
            Ok(Err(e)) => return HttpResponse::InternalServerError().body(e.to_string()),
            Err(_) => return HttpResponse::RequestTimeout().body("Insert en roles demoró demasiado."),
        };
        if i == 6 {
            let _insert_personal = match timeout(Duration::from_secs(5), async {
                sqlx::query("INSERT INTO personal_data (user_id, full_name, birth_date, address, phone_number) VALUES (?, ?, ?, ?, ?)")
                    .bind(user_id)
                    .bind("valentino grande")
                    .bind("2024-07-18 15:30:00")
                    .bind("santa coloma 9282")
                    .bind("543412115831")
                    .execute(pool.get_ref())
                    .await
            })
            .await
            {
                Ok(Ok(_)) => (),
                Ok(Err(e)) => return HttpResponse::InternalServerError().body(e.to_string()),
                Err(_) => return HttpResponse::RequestTimeout().body("Insert en personal_data demoró demasiado."),
            };
        }
    }

    HttpResponse::Created().finish()
}
