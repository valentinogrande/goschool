use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};


#[get("/api/v1/register_testing_users/")]
pub async fn register_users(pool: web::Data<MySqlPool>) -> impl Responder{
    let users: Vec<&str> = Vec::from(["admin","student","preceptor","father","teacher"]);
    for user in users.iter() {
        let hash = hash(user, DEFAULT_COST).unwrap();
        let res = match sqlx::query("INSERT INTO users (password, email, role) VALUES (?,?,?)")
        .bind(&hash)
        .bind(user)
        .bind(user)
        .execute(pool.get_ref()).await{
            Ok(_) => {},
            Err(e) => {}
        };
    }
    HttpResponse::Created().finish()
}
