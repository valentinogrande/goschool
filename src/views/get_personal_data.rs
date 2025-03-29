use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use sqlx::Row;


use crate::{RespondPersonalData, validate};

#[utoipa::path(
    get,
    path = "/api/v1/get_personal_data/",
    request_body(content = String, description = "JWT token", content_type = "application/json"),
    responses(
        (status = 200, description = "Success", body = RespondPersonalData),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
#[get("/api/v1/get_personal_data/")]
async fn get_personal_data(pool: web::Data<MySqlPool>, req: HttpRequest) -> impl Responder {
    
    let option_jwt = req.cookie("jwt");
    let jwt: String;
    if let Some(jsonwt) = option_jwt {
        jwt = jsonwt.value().to_string();
    }else {
        return HttpResponse::Unauthorized().json("Missing Json web token");
    }
    let decode = validate(jwt.clone());

     match decode{
        Ok(token_data) => {
            let userid = token_data.claims.subject as i32;
            match sqlx::query(
                "SELECT nombre_completo, edad, mensaje FROM person WHERE user_id = ?")
                .bind(userid)
                .fetch_one(pool.get_ref())
                .await {
                    Ok(record) => {
                        let name = record.get::<String, &str>("nombre_completo");
                        let age = record.get::<i32, &str>("edad");
                        let msg = record.get::<String, &str>("mensaje");
                        let personal_data = RespondPersonalData::new(name, age, msg);
                        HttpResponse::Ok().json(personal_data)
                    },
                    Err(_) => HttpResponse::NotFound().json("Personal data not found")
                }
        },
        Err(e) => HttpResponse::Unauthorized().json(e.to_string())
    }
}
