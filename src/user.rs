use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize)]
pub struct User{
    pub userid: i32,
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewUser{
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials{
    pub email: String,
    pub password: String,
}

impl NewUser{
    pub fn new(username: String,password: String, email: String) -> NewUser {
        NewUser{
            username,
            password,
            email,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PersonalData {
    pub jwt: String,
    pub nombre_completo: String,
    pub edad: i32,
    pub mensaje: String,
}

#[derive(Serialize, Deserialize)]
pub struct RespondPersonalData{
    pub nombre_completo: String,
    pub edad: i32,
    pub mensaje: String,
}

impl RespondPersonalData{
    pub fn new(nombre_completo: String, edad: i32, mensaje: String) -> RespondPersonalData{
        RespondPersonalData{
            nombre_completo,
            edad,
            mensaje,
        }
    }
}
