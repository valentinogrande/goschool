use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::user::NewTeacherData;
use crate::jwt::validate;

#[utoipa::path(
    post,
    path = "/api/v1/update_teachers/",
    request_body(content = NewTeacherData, description = "update student info", content_type = "application/json"),
    responses(
        (status = 201, description = "personal data of teachers were updated successfully"),
        (status = 500, description = "Internal server error")
    )
)]

#[post("/api/v1/update_teachers/")]
pub async fn update_teachers(
    pool: web::Data<MySqlPool>,
    user: web::Json<NewTeacherData>,
    req: HttpRequest,
) -> impl Responder {
    let jwt = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("No JWT provided"),
    };

    let token = match validate(jwt.value().to_string()) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("JWT validation failed: {:?}", err);
            return HttpResponse::Unauthorized().json("Invalid JWT");
        }
    };

    let user_id = token.claims.subject;

    let roles = sqlx::query_as::<_, (bool, bool)>(
        "SELECT is_teacher, is_admin FROM users WHERE id = ?",
    )
    .bind(user_id as i32)
    .fetch_one(pool.get_ref())
    .await;

    let (is_teacher, is_admin) = match roles {
        Ok(role) => role,
        Err(err) => {
            eprintln!("DB error while checking roles: {:?}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !is_teacher && !is_admin {
        return HttpResponse::Unauthorized().json("Not authorized");
    }

    let update_result = sqlx::query(
        r#"
        INSERT INTO teachers (user_id, subject, grades)
        VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE
            subject = VALUES(subject),
            grades = VALUES(grades)
        "#,
    )
    .bind(user_id as i32)
    .bind(&user.subject)
    .bind(&user.grades)
    .execute(pool.get_ref())
    .await;

    match update_result {
        Ok(_) => HttpResponse::Created().json("Teacher updated successfully"),
        Err(err) => {
            eprintln!("Database error while updating teacher: {:?}", err);
            HttpResponse::InternalServerError().json("Failed to update teacher")
        }
    }
}
