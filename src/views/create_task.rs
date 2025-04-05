use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct NewTask {
    task: String,
    grade: i32,
}

#[utoipa::path(
    post,
    path = "/api/v1/create_task/",
    request_body(content = NewTask, description = "task creation data", content_type = "application/json"),
    responses(
        (status = 201, description = "task created successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "bad request"),

    )
)]
#[post("/api/v1/create_task/")]
pub async fn create_task(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    task: web::Json<NewTask>,
) -> impl Responder {
    let jwt = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(jwt.value().to_string()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.subject;

    let roles = sqlx::query_as::<_, (bool, bool)>(
        "SELECT is_teacher, is_admin FROM users WHERE id = ?",
    )
    .bind(user_id as i32)
    .fetch_one(pool.get_ref())
    .await;

    let (is_teacher, is_admin) = match roles {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if !is_teacher && !is_admin {
        return HttpResponse::Unauthorized().finish();
    }

    let insert_result = sqlx::query("INSERT INTO tasks (task, grade, teacher) VALUES (?, ?, ?)")
        .bind(&task.task)
        .bind(task.grade)
        .bind(user_id as i32)
        .execute(pool.get_ref())
        .await;

    match insert_result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
