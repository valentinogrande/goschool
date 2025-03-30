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
pub async fn update_teachers(pool: web::Data<MySqlPool>, user: web::Json<NewTeacherData>, req: HttpRequest) -> impl Responder {
    let jwt = req.cookie("jtk");
    if let Some(jwt_val) = jwt {
        let validate = validate(jwt_val.value().to_string());
        if let Ok(res) = validate {

            let result = sqlx::query("UPDATE students SET subject WHERE id = ?")
                .bind(user.subject.clone())
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
