
use actix_web::web;
use sqlx::{MySqlPool, Row};

pub async fn get_admins(pool: &web::Data<MySqlPool>) -> Result<Vec<i32>, sqlx::Error> {
    let result = sqlx::query("SELECT id FROM users WHERE is_admin = 1")
        .fetch_all(pool.get_ref())
        .await?;

    let admins = result.into_iter()
        .map(|row| row.get::<i32, _>("id")) // 💡 Usamos `get::<T, &str>()`
        .collect();

    Ok(admins)
}
