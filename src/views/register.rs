use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};

use crate::{jwt::validate, NewUser};


#[utoipa::path(
    post,
    path = "/api/v1/register/",
    request_body(content = NewUser, description = "User registration data", content_type = "application/json"),
    responses(
        (status = 201, description = "User created successfully"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/api/v1/register/")]
pub async fn create_user(pool: web::Data<MySqlPool>, user: web::Json<NewUser>, req: HttpRequest) -> impl Responder {
    let hashed_pass = hash(&user.password, DEFAULT_COST);
    if let Err(e) = hashed_pass {
        return HttpResponse::InternalServerError().json(e.to_string())
    }
    else {
        let hashed_pass = hashed_pass.unwrap();
        let mut result = sqlx::query("INSERT INTO users (password, email) VALUES (?,?)")
            .bind(&hashed_pass)
            .bind(user.email.clone());

        let adm = req.cookie("jwt");
        if let Some(cookie) = adm {
            let jwt_val = validate(cookie.value().to_string());
            if let Ok(res) = jwt_val {
                if res.claims.subject == 2 { // important this is the admin id
                    result = sqlx::query("INSERT INTO users (password, email, is_teacher) VALUES (?,?,?)")
                    .bind(&hashed_pass)
                    .bind(user.email.clone())
                    .bind(user.is_teacher);
                }
            }
        }
       
        match result.execute(pool.get_ref()).await {
            Ok(_) => HttpResponse::Created().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}
