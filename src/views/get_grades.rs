use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use serde::{Serialize, Deserialize};

use crate::jwt::validate;
use crate::filters::GradeFilter;


#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GradeType {
    Numerical,
    Conceptual,
    Percentage,
}

#[get("/api/v1/get_student_grades/")]
pub async fn get_grades(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<GradeFilter>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };
    
    let user = token.claims.user;  

    let grades = match user.get_grades(pool, Some(filter.into_inner())).await { 
        Ok(g) => g,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(grades)
}
