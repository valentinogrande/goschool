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
    let jwt = req.cookie("jtk");
    
    if jwt.is_none() {
        return HttpResponse::Unauthorized().json("No JWT provided");
    }

    // Validar el JWT
    let validate = validate(jwt.unwrap().value().to_string());
    if let Err(err) = validate {
        eprintln!("JWT validation failed: {:?}", err);
        return HttpResponse::Unauthorized().json("Invalid JWT");
    }

    let res = validate.unwrap();

    let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM teachers WHERE user_id = ?)")
        .bind(res.claims.subject as i32)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or((false,));

    if !exists.0 {
        return HttpResponse::Unauthorized().json("User is not a teacher");
    }

    let result = sqlx::query(
        "INSERT INTO teachers (user_id, subject, grades) VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE subject = VALUES(subject), grades = VALUES(grades)"
    )
    .bind(res.claims.subject as i32)
    .bind(&user.subject)
    .bind(&user.grades)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Created().json("Teacher updated successfully"),
        Err(err) => {
            eprintln!("Database error: {:?}", err);
            HttpResponse::InternalServerError().json("Failed to update teacher")
        }
    }
}
