use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use serde::{Serialize, Deserialize};
use sqlx::QueryBuilder;

use crate::jwt::validate;
use crate::structs::{Role, Grade};
use crate::filters::{UserFilter, GradeFilter};


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

    // importante:
    // esto te deberia dar todas las grades disponebles por usuario, despues se 
    // deberia poder filtar por el usuario que interesa.

    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };
    
    let user = token.claims.user;  

    let student_id = match filter.student_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json("Missing student_id"),
    };

    if user.role != Role::teacher{}
    let students_id = match user.get_students(pool.clone(), None).await
        {
            Ok(r) => r,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };
    
    if !students_id.contains(&student_id) {
            return HttpResponse::Unauthorized().json("Not authorized to access this student's data");
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
