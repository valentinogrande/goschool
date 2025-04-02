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

#[derive(Serialize, Deserialize, ToSchema)]
pub struct StudentData {
    pub id: i32,
    pub user_id: i32,
    pub grade: i32,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NewStudentData {
    pub grade: i8,
    pub divition: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TeacherData{
    pub id: i32,
    pub user_id: i32,
    pub grades: String,
    pub subject: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NewTeacherData{
    pub subject: String,
    pub grades: String,
}
