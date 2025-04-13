use actix_web::{get, web, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use bcrypt::{hash, DEFAULT_COST};


#[get("/api/v1/register_testing_users/")]
pub async fn register_users(pool: web::Data<MySqlPool>) -> impl Responder{
    let users: Vec<&str> = Vec::from(["admin","student","preceptor","father","teacher"]);
    for (i, user) in users.iter().enumerate() {
        let hash = hash(user, DEFAULT_COST).unwrap();
        let _res = match sqlx::query("INSERT INTO users (password, email) VALUES (?,?)")
        .bind(&hash)
        .bind(user)
        .execute(pool.get_ref()).await{
            Ok(_) => {},
            Err(_) => {}
        };

        let i = i +1;
        log::info!("user: {:?}, id : {}", user,i);
        let _res = match sqlx::query("INSERT INTO roles (user_id, role) VALUES (?,?)")
        .bind(i as i32)
        .bind(user)
        .execute(pool.get_ref()).await{
            Ok(r) => r,
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
        };
    }
    HttpResponse::Created().finish()
}
