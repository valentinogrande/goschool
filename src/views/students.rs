use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::user::NewStudentData;
use crate::jwt::validate;

#[utoipa::path(
    post,
    path = "/api/v1/update_students/",
    request_body(content = NewStudentData, description = "update student info{
the id is proportionated by the jwt.
}", content_type = "application/json"),
    responses(
        (status = 201, description = "personal data of students were updated successfully"),
        (status = 500, description = "Internal server error")
    )
)]

#[post("/api/v1/update_students/")]
pub async fn update_students(
    pool: web::Data<MySqlPool>,
    user: web::Json<NewStudentData>,
    req: HttpRequest,
) -> impl Responder {
    let jwt = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(jwt.value().to_string()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let grade_lookup = sqlx::query_as::<_, (i32,)>(
        "SELECT id FROM grades WHERE year = ? AND divition = ?",
    )
    .bind(user.grade)
    .bind(&user.divition)
    .fetch_one(pool.get_ref())
    .await;

    let grade_id = match grade_lookup {
        Ok((id,)) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let result = sqlx::query("INSERT INTO students (user_id, grade_id) VALUES (?, ?)")
        .bind(token.claims.subject as i32)
        .bind(grade_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
