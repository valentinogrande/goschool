use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use chrono::{Datelike, NaiveDate};
use chrono::{DateTime, Utc};
use sqlx::QueryBuilder;

use crate::user::Role;
use crate::jwt::validate;
use crate::views::create_assessment::AssessmentType;

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
#[derive(serde::Serialize, serde::Deserialize)]
struct AssessmentFilter{
    subject_id: Option<u64>,
    task: Option<String>,
    due: Option<bool>,
}

#[get("/api/v1/get_student_assessments/{student_id}/")]
pub async fn get_assessments(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    student_id: web::Path<u64>,
    filter: web::Query<AssessmentFilter>,
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
    let mut student_id = student_id.into_inner();
    if student_id == 0 {
        student_id = token_id;
    }

    let role = token.claims.role;
    
    if role == Role::father {
            let students_id: Vec<u64> = match sqlx::query_scalar("SELECT student_id FROM families WHERE father_id = ?")
                .bind(token_id)
                .fetch_all(pool.get_ref())
                .await
            {
                Ok(r) => r,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };
            
            if !students_id.contains(&student_id) {
                return HttpResponse::Unauthorized().json("Not authorized to access this student's data");
            }
        }
    else if role == Role::student {
        if student_id != token_id {
            return HttpResponse::Unauthorized().finish();
        }
    }
    else if  role == Role::admin || role == Role::teacher {}
    else {
        return HttpResponse::Unauthorized().finish();
    }
        
    let mut builder = QueryBuilder::new(
        r#"
        SELECT a.* 
        FROM assessments a
        JOIN subjects s ON a.subject_id = s.id
    "#);

    if role != Role::teacher {
        builder.push(
            r#"
            JOIN users u ON s.course_id = u.course_id
            WHERE u.id = "#,
        )
        .push_bind(student_id);
    } else {
        builder.push("WHERE s.teacher_id = ")
               .push_bind(token_id);
    }

if let Some(subject_id) = filter.subject_id {
    builder.push(" AND a.subject_id = ");
    builder.push_bind(subject_id);
}
if let Some(task) = &filter.task {
    builder.push(" AND a.task LIKE ");
    builder.push_bind(format!("%{}%", task));
}
if let Some(due) = filter.due {
    if due {
        builder.push(" AND a.due_date >= ");
        builder.push_bind(NaiveDate::from_ymd_opt(Utc::now().year(), Utc::now().month(), Utc::now().day()).unwrap());
    }
}

let query = builder.build_query_as::<Assessment>();

let assessments: Vec<Assessment> = match query
    .fetch_all(pool.get_ref())
    .await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
};
    HttpResponse::Ok().json(assessments)

}
