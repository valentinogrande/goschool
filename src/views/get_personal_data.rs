use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use chrono::NaiveDate;

use crate::jwt::validate;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, sqlx::Decode)]
struct PersonalData {
    full_name: String,
    phone_number: String,
    address: String,
    birth_date: NaiveDate,  
}

#[get("/api/v1/get_personal_data/")]
pub async fn get_personal_data(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };

    let token_id = token.claims.subject as u64;
    
    let personal_data = match sqlx::query_as::<_, PersonalData>("SELECT full_name, phone_number, address, birth_date FROM personal_data WHERE user_id = ?")
        .bind(token_id)
        .fetch_one(pool.get_ref())
        .await {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    HttpResponse::Ok().json(personal_data)
}
