use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "ENUM('admin', 'teacher', 'student', 'preceptor', 'father')")]
#[serde(rename_all = "lowercase")]
#[schema(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Teacher,
    Student,
    Preceptor,
    Father,
}

#[derive(Serialize,Deserialize,ToSchema)]
pub struct User{
    pub id: i32,
    pub password: String,
    pub email: String,
    pub role: Role,
    pub last_login: String,
}

#[derive(Serialize, Deserialize,ToSchema)]
pub struct NewUser{
    pub password: String,
    pub email: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize,ToSchema)]
pub struct Credentials{
    pub email: String,
    pub password: String,
}
