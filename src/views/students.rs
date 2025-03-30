use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::user::NewStudentData;
use crate::jwt::validate;

#[utoipa::path(
    post,
    path = "/api/v1/update_students/",
    request_body(content = NewStudentData, description = "update student info", content_type = "application/json"),
    responses(
        (status = 201, description = "personal data of students were updated successfully"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/api/v1/update_students/")]
pub async fn update_students(pool: web::Data<MySqlPool>, user: web::Json<NewStudentData>, req: HttpRequest) -> impl Responder {
    let jwt = req.cookie("jtk");
    if let Some(jwt_val) = jwt {
        let validate = validate(jwt_val.value().to_string());
        if let Ok(res) = validate {

            let result = sqlx::query("UPDATE students SET grade = ?, divition = ? WHERE id = ?")
                .bind(user.grade)
                .bind(user.divition.clone())
                .bind(res.claims.subject as i32)
                .execute(pool.get_ref())
                .await;

            HttpResponse::Created().finish()

        }
        else {
            return HttpResponse::Unauthorized().finish();
        }
    }else {
        HttpResponse::Unauthorized().finish()
    }
}
