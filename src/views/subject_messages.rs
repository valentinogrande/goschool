use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post, put, delete};
use sqlx::mysql::MySqlPool;
use actix_multipart::Multipart;

use crate::jwt::validate;
use crate::traits::{Post, Get, Update, Delete};
use crate::filters::SubjectMessageFilter;
use crate::structs::UpdateSubjectMessage;

#[get("/api/v1/subject_messages/")]
pub async fn get_subject_messages(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    filter: web::Query<SubjectMessageFilter>) -> impl Responder {

    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;
    
    let messages = match user.get_subject_messages(&pool, filter.into_inner()).await {
        Ok(m) => m,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(messages)
}

#[post("/api/v1/subject_messages/")]
pub async fn post_subject_message(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    multipart: Multipart
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    
    let user = token.claims.user;
        
    user.post_subject_messages(&pool, multipart).await
}

#[put("/api/v1/subject_messages/{id}")]
pub async fn update_subject_message(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
    data: web::Json<UpdateSubjectMessage>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    let user = token.claims.user;
    user.update_subject_message(pool.get_ref(), *id, data.into_inner()).await
}

#[delete("/api/v1/subject_messages/{id}")]
pub async fn delete_subject_message(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<u64>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    let user = token.claims.user;
    user.delete_subject_message(pool.get_ref(), *id).await
}
