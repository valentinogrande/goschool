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
pub async fn update_students(pool: web::Data<MySqlPool>, user: web::Json<NewStudentData>, req: HttpRequest) -> impl Responder {
    let jwt = req.cookie("jwt");
    if let Some(jwt_val) = jwt {
        let validate = validate(jwt_val.value().to_string());
        if let Ok(res) = validate {
            let result: Result<(i32,), sqlx::Error> = sqlx::query_as("SELECT * FROM grades WHERE year = ? AND divition = ?")
                .bind(user.grade)
                .bind(&user.divition)
                .fetch_one(pool.get_ref())
                .await;
            if let Ok(result) = result{
                let course = result.0;
                let result = sqlx::query("INSERT INTO students (user_id,grade_id) values (?,?)")
                    .bind(res.claims.subject as i32)
                    .bind(course)
                    .execute(pool.get_ref())
                    .await;
                match result{
                    Ok(_) => return HttpResponse::Created().finish(),
                    Err(_) => return HttpResponse::InternalServerError().finish(),
                }
            }else {
                return HttpResponse::InternalServerError().finish();
            }

        }
        else {
            return HttpResponse::Unauthorized().finish();
        }
    }else {
        HttpResponse::Unauthorized().finish()
    }
}
