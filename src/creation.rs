use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize,Serialize};
use sqlx::mysql::MySqlPool;
use utoipa::ToSchema;

use crate::jwt::validate;
use crate::functions::get_admins;

#[derive(Serialize,Deserialize,ToSchema)]
pub struct Grades{
    primary: u8,
    secondary: u8,
    divitions: u8,
}


#[utoipa::path(
    post,
    path = "/api/v1/create_grade/",
    request_body(content = Grades, description = "create table of grades", content_type = "application/json"),
    responses(
        (status = 201, description = "User created successfully"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/api/v1/create_grade/")]
pub async fn create_grades(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    grades: web::Json<Grades>,
) -> impl Responder {
    // Validar JWT desde la cookie
    let jwt_cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let validation = validate(jwt_cookie.value().to_string());
    let Ok(res) = validation else {
        return HttpResponse::Unauthorized().finish();
    };

    // Verificar si el usuario es administrador
    let Ok(admins) = get_admins(&pool).await else {
        return HttpResponse::InternalServerError().finish();
    };

    if !admins.contains(&(res.claims.subject as i32)) {
        return HttpResponse::Unauthorized().finish();
    }

    // Crear tabla grades si no existe
    let create_table = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS grades (
            id INT AUTO_INCREMENT PRIMARY KEY,
            year INT NOT NULL,
            divition VARCHAR(25)
        )
        "#,
    )
    .execute(pool.get_ref())
    .await;

    if create_table.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    // Insertar datos en grades
    let mut error_count = 0;
    let total_years = grades.primary + grades.secondary;

    for year in 1..=total_years {
        for divition in 1..=grades.divitions {
            let insert = sqlx::query("INSERT INTO grades (year, divition) VALUES (?, ?)")
                .bind(year)
                .bind(divition.to_string())
                .execute(pool.get_ref())
                .await;

            if insert.is_err() {
                error_count += 1;
            }
        }
    }

    if error_count == 0 {
        HttpResponse::Created().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
