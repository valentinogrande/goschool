use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::user::Roles;

#[derive(Debug, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "ENUM('exam','homework','project')")]
#[serde(rename_all = "lowercase")]
pub enum AssessmentType {
    Exam,
    Homework,
    Project,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewTask {
    subject: u64,
    task: String,
    due_date: String,
    #[serde(rename = "type")]
    type_: AssessmentType,
}


#[post("/api/v1/create_assessment/")]
pub async fn create_assessment(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    task: web::Json<NewTask>,
) -> impl Responder {
    let jwt = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(jwt.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.subject as u64;

    let roles = match crate::sqlx_fn::get_roles(&pool, user_id).await {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if !(roles.contains(&Roles::new("teacher".to_string())) || roles.contains(&Roles::new("admin".to_string()))) {
        return HttpResponse::BadRequest().finish();
    }

    // checking that the teacher is the owner of the subject
    let teacher_subject: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM subjects WHERE teacher_id = ? AND id = ?)")
        .bind(user_id)
        .bind(task.subject)
        .fetch_one(pool.get_ref())
        .await {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if !teacher_subject {
        return HttpResponse::Unauthorized().finish();
    }
        let insert_result = sqlx::query("INSERT INTO assessments (task, subject_id, type, due_date) VALUES (?, ?, ?, ?)")
        .bind(&task.task)
        .bind(task.subject)
        .bind(&task.type_)
        .bind(&task.due_date)
        .execute(pool.get_ref())
        .await;

    match insert_result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}
