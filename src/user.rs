use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize,Deserialize,ToSchema)]
pub struct User{
    pub id: i32,
    pub password: String,
    pub email: String,
    pub role: String,
    pub last_login: String,
}

#[derive(Serialize, Deserialize,ToSchema)]
pub struct NewUser{
    pub password: String,
    pub email: String,
    pub role: String,
}

#[derive(Serialize, Deserialize,ToSchema)]
pub struct Credentials{
    pub email: String,
    pub password: String,
}
