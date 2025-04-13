use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;
use chrono::{DateTime, Utc};

use crate::sqlx_fn;
use crate::user::Roles;
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
    id: u64,
    subject_id: u64,
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
    student_id: web::Path<u64>,
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
    
    if !(roles.contains(&Roles::new("admin".to_string()))){
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
        }else{
            return HttpResponse::Unauthorized().finish()   
        }
    }
    
    
    let assessments = match sqlx::query_as::<_, Assessment>(
        r#"
        SELECT a.* 
        FROM assessments a
        JOIN subjects s ON a.subject_id = s.id
        JOIN users u ON s.course_id = u.course_id
        WHERE u.id = ?
        "#
    )
    .bind(student_id)
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

    let user_id = token.claims.subject as u64;
    
    let roles = match crate::sqlx_fn::get_roles(&pool, user_id).await {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if !(roles.contains(&Roles::new("student".to_string()))){
        return HttpResponse::Unauthorized().finish();
    }
        
    let assessments = match sqlx::query_as::<_, Assessment>(
        r#"
        SELECT a.* 
        FROM assessments a
        JOIN subjects s ON a.subject_id = s.id
        JOIN users u ON s.course_id = u.course_id
        WHERE u.id = ?
        "#
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(assessments)
}
