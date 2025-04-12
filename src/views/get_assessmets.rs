use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use sqlx::QueryBuilder;
use chrono::NaiveDate;
use chrono::{DateTime, Utc};


use crate::jwt::validate;

#[derive(Debug, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "ENUM('exam','homework','project')")]
#[sqlx(rename_all = "lowercase")] 
#[serde(rename_all = "lowercase")]
pub enum AssessmentType {
    Exam,
    Homework,
    Project,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
struct Assessment {
    id: i64,
    subject_id: i64,
    task: String,
    due_date: NaiveDate,  
    created_at: DateTime<Utc>,
    #[sqlx(rename = "type")] 
    #[serde(rename = "type")]
    type_: AssessmentType,
}

#[get("/api/v1/get_student_assessments_by_id/{student_id}/")]
pub async fn get_assessments_by_id(
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
    
    let course_id = match sqlx::query_scalar::<_, i64>("SELECT course_id FROM users WHERE id = ?")
        .bind(student_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(c) => c,
        Err(_) => return HttpResponse::BadRequest().json("Invalid student id"),
    };
    
    let subjects: Vec<i64> = match sqlx::query_scalar("SELECT id FROM subjects WHERE course_id = ?")
        .bind(course_id)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if subjects.is_empty() {
        return HttpResponse::BadRequest().json("No subjects found on this course")
    }

    let mut builder = QueryBuilder::new("SELECT * FROM assessments WHERE subject_id IN (");
    let mut separated = builder.separated(", ");
    for id in &subjects {
        separated.push_bind(id);
    }
    builder.push(")");

    let assessments = match builder.build_query_as::<Assessment>()
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(assessments)
}

#[get("/api/v1/get_student_assessments/")]
pub async fn get_assessments(
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
        return HttpResponse::Unauthorized().json("Not authorized");
    }
    
    let course_id = match sqlx::query_scalar::<_, i64>("SELECT course_id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(c) => c,
        Err(_) => return HttpResponse::BadRequest().json("Invalid student id"),
    };
    
    let subjects: Vec<i64> = match sqlx::query_scalar("SELECT id FROM subjects WHERE course_id = ?")
        .bind(course_id)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if subjects.is_empty() {
        return HttpResponse::BadRequest().json("No subjects found on this course")
    }

    let mut builder = QueryBuilder::new("SELECT * FROM assessments WHERE subject_id IN (");
    let mut separated = builder.separated(", ");
    for id in &subjects {
        separated.push_bind(id);
    }
    builder.push(")");

    let assessments = match builder.build_query_as::<Assessment>()
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(assessments)
}
