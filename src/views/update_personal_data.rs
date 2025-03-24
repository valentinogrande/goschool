use actix_web::{post, web, HttpResponse,  Responder};
use sqlx::mysql::MySqlPool;

use crate::{PersonalData, validate};


#[utoipa::path(
    post,
    path = "/api/v1/update_personal_data/",
    request_body(content = PersonalData, description = "Personal data to update", content_type = "application/json"),
    responses(
        (status = 201, description = "Personal data updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/api/v1/update_personal_data/")]
async fn update_personal_data(
    pool: web::Data<MySqlPool>,
    data: web::Json<PersonalData>) -> impl Responder {
        
    let decode = validate(data.jwt.clone());

    match decode {
        Ok(token_data) => {
            let userid = token_data.claims.subject as i32;
            match sqlx::query(
                "INSERT INTO person (nombre_completo, edad, mensaje, user_id) VALUES (?,?,?,?)")
                .bind(data.nombre_completo.clone())
                .bind(data.edad)
                .bind(data.mensaje.clone())
                .bind(userid)
                .execute(pool.get_ref())
                .await {
                    Ok(_) => HttpResponse::Created().json("Personal data was updated"),
                    Err(e) => HttpResponse::InternalServerError().json(e.to_string())
                }
        },
        Err(e) => HttpResponse::Unauthorized().json(e.to_string())
    }
}
