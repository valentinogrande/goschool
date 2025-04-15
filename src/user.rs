use serde::{Serialize, Deserialize};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "ENUM('admin', 'teacher', 'student', 'preceptor', 'father')")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    admin,
    teacher,
    student,
    preceptor,
    father,
}



#[derive(Serialize,Deserialize)]
pub struct User{
    pub id: u64,
    pub password: String,
    pub email: String,
    pub role: Role,
    pub last_login: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewUser{
    pub password: String,
    pub email: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials{
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct CredentialsRole{
    pub email: String,
    pub password: String,
    pub role: Role,
}
