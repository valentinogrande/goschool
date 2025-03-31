use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize,Serialize};
use sqlx::mysql::MySqlPool;
use utoipa::ToSchema;

use crate::jwt::validate;
use crate::functions::get_admins;

#[derive(Serialize,Deserialize,ToSchema)]
pub struct Grades{
    primary: u8,
    secondary: u8,
    divitions: u8,
}


#[utoipa::path(
    post,
    path = "/api/v1/create_grade/",
    request_body(content = Grades, description = "create table of grades", content_type = "application/json"),
    responses(
        (status = 201, description = "User created successfully"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/api/v1/create_grade/")]
pub async fn create_grades(pool: web::Data<MySqlPool>, req: HttpRequest, grades: web::Json<Grades>) -> impl Responder {
    let cookies = req.cookie("jwt");
    if let Some(jwt) = cookies {
        let val = validate(jwt.value().to_string());
            match val {
            Ok(res) => {
                let admins = get_admins(&pool).await;
               
                if admins.unwrap().contains(&(res.claims.subject as i32)) {
                    let result = sqlx::query("CREATE TABLE IF NOT EXISTS grades(
                        id INT AUTO_INCREMENT,
                        year INT NOT NULL,
                        divition VARCHAR(25),
                        PRIMARY KEY (id)
                        )")
                        .execute(pool.get_ref()).await;
                    
                    if let Ok(_) = result{
                        let mut error = 0;
                        for i in 1..=(grades.primary+grades.secondary) {             
                            for j in 1..=grades.divitions{
                                let result = 
                                sqlx::query("INSERT INTO grades(year, divition)VALUES(?,?)")
                                    .bind(i)
                                    .bind(j)
                                    .execute(pool.get_ref())
                                    .await;
                                if let Err(_) = result {
                                    error=error+1; 
                                }
                            }
                        }
                        if error == 0 {
                            return HttpResponse::Created().finish();
                        }
                        else{
                            return HttpResponse::InternalServerError().finish();
                        }
                    }else {
                        return HttpResponse::InternalServerError().finish();
                    }
                }else {
                    HttpResponse::Unauthorized().finish()
                }
            },
            Err(_) => {HttpResponse::Unauthorized().finish()}
        }
    }
    else {
        HttpResponse::Unauthorized().finish()
    }
}
