use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;

#[derive(serde::Serialize, serde::Deserialize)]
struct NewFamily {
    student_id: i64,
    father_id: i64,
}

#[post("/api/v1/post_families/")]
pub async fn post_families(pool: web::Data<MySqlPool>, req: HttpRequest, family: web::Json<NewFamily>) -> impl Responder{
        let jwt = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let token = match validate(jwt.value()) {
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

    if !is_admin {
        return HttpResponse::Unauthorized().finish();
    }
    
    let query = sqlx::query("INSERT INTO families (student_id, father_id) VALUES (? ,?)")
        .bind(family.student_id)
        .bind(family.father_id)
        .execute(pool.get_ref())
        .await;
    
    match query {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}


