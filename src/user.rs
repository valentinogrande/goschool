use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize,Deserialize,ToSchema)]
pub struct User{
    pub id: i32,
    pub password: String,
    pub email: String,
    pub is_admin: bool,
    pub is_teacher: bool,
}

#[derive(Serialize, Deserialize,ToSchema)]
pub struct NewUser{
    pub password: String,
    pub email: String,
    pub is_admin: i8,
    pub is_teacher: i8,
}

#[derive(Serialize, Deserialize,ToSchema)]
pub struct Credentials{
    pub email: String,
    pub password: String,
}
