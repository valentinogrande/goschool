use actix_web::{post, web, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::verify;

use crate::structs::{Role, Credentials};

#[post("/api/v1/roles/")]
pub async fn get_roles(
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

    let roles:Vec<Role> = match sqlx::query_scalar::<_,Role>("SELECT role FROM roles WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(pool.get_ref())
        .await {
        Ok(r) => r,
        Err(e) => return HttpResponse::Unauthorized().body(e.to_string()),
    };
    
    HttpResponse::Ok().json(roles)   
}
