use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use rust_decimal::Decimal;
use crate::sqlx_fn;
use crate::user::Roles;
use sqlx::QueryBuilder;

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
    pub id: u64,
    pub description: Option<String>,
    pub grade: Decimal,
    pub student_id: u64,
    pub subject_id: u64,
    pub assessment_id: Option<u64>,
    pub grade_type: Option<GradeType>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
struct GradeFilter{
    subject_id: Option<u64>,
    description: Option<String>,
}


#[get("/api/v1/get_student_grades_by_id/{student_id}/")]
pub async fn get_grades_by_id(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    student_id: web::Path<u64>,
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

    let user_id = token.claims.subject as u64;
    let student_id = student_id.into_inner();
    
    let roles = match sqlx_fn::get_roles(&pool, user_id).await {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if !(roles.contains(&Roles::new("admin".to_string()))) {
        if roles.contains(&Roles::new("father".to_string())) {
            let students_id: Vec<u64> = match sqlx::query_scalar("SELECT student_id FROM families WHERE father_id = ?")
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
        } else {
            return HttpResponse::Unauthorized().finish();
        }
    }

    let mut query = QueryBuilder::new("SELECT * FROM grades WHERE student_id = ");
    query.push_bind(student_id);

    if let Some(subject_id) = filter.subject_id {
        query.push(" AND subject_id = ");
        query.push_bind(subject_id);
    }

    if let Some(ref description) = filter.description {
        query.push(" AND description LIKE ");
        query.push_bind(format!("%{}%", description));
    }
    

    let grades: Vec<Grade> = match query
        .build_query_as()
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(grades)
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

    let user_id = token.claims.subject as u64;
    
    let mut query = QueryBuilder::new("SELECT * FROM grades WHERE student_id = ");
    query.push_bind(user_id);

    if let Some(subject_id) = filter.subject_id {
        query.push(" AND subject_id = ");
        query.push_bind(subject_id);
    }


    if let Some(ref description) = filter.description {
        query.push(" AND description LIKE ");
        query.push_bind(format!("%{}%", description));
    }

    let grades: Vec<Grade> = match query
        .build_query_as()
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(grades)
}
