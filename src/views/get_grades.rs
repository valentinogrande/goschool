use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use rust_decimal::Decimal;


use crate::jwt::validate;

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GradeType {
    Numerical,
    Conceptual,
    Percentage,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Grade {
    pub id: i64,
    pub description: Option<String>,
    pub grade: Decimal,
    pub student_id: i64,
    pub subject_id: i64,
    pub assessment_id: Option<i64>,
    pub grade_type: Option<GradeType>,
    pub created_at: Option<DateTime<Utc>>,
}


#[get("/api/v1/get_student_grades_by_id/{student_id}/")]
pub async fn get_grades_by_id(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    student_id: web::Path<i64>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };

    let user_id = token.claims.subject as i64;
    let student_id = student_id.into_inner();
    
    let role = match sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    
    if role == "father" {
        let students_id: Vec<i64> = match sqlx::query_scalar("SELECT student_id FROM families WHERE father_id = ?")
            .bind(user_id)
            .fetch_all(pool.get_ref())
            .await
        {
            Ok(r) => r,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };
        
        if !students_id.contains(&student_id) {
            return HttpResponse::Unauthorized().json("Not authorized to access this student's data");
        }
    } else if role != "admin" && user_id != student_id {
        return HttpResponse::Unauthorized().json("Not authorized to access this student's data");
    }
    

    let grades: Vec<Grade> = match sqlx::query_as::<_, Grade>("SELECT * from grades WHERE student_id = ?")
        .bind(student_id)
        .fetch_all(pool.get_ref())
        .await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    HttpResponse::Ok().json(grades)
}

#[get("/api/v1/get_student_grades/")]
pub async fn get_grades(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };

    let user_id = token.claims.subject as i64;
    
    let role = match sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    
     if role != "student" {
        return HttpResponse::Unauthorized().json("Not authorized to access this student's data");
    }

    let grades: Vec<Grade> = match sqlx::query_as::<_, Grade>("SELECT * from grades WHERE student_id = ?")
        .bind(user_id)
        .fetch_all(pool.get_ref())
        .await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    HttpResponse::Ok().json(grades)
}
