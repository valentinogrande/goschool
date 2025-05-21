use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use rust_decimal::Decimal;
use sqlx::QueryBuilder;

use crate::jwt::validate;
use crate::user::Role;

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


#[get("/api/v1/get_grade_selfassessable/{selfassessable_id}/")]
pub async fn get_grade_selfassessable(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    selfassessable_id: web::Path<u64>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };

    let token_id = token.claims.subject as u64;
   
    let role = token.claims.role;
   
    todo!();
    if role == Role::father{

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
