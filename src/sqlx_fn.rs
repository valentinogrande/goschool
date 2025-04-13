use sqlx::mysql::MySqlPool;

use crate::user::Roles;

pub async fn get_roles(pool: &MySqlPool, user_id: u64) -> Result<Vec<Roles>, sqlx::Error>{
    let roles: Vec<Roles> = sqlx::query_as::<_, Roles>("SELECT role FROM roles WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(roles)
}
