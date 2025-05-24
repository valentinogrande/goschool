use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::filters::{AssessmentFilter, SubjectFilter, UserFilter};


#[get("/api/v1/get_student_assessments/")]
pub async fn get_assessments(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<AssessmentFilter>,
    subject_filter: web::Query<SubjectFilter>,
    person_filter: web::Query<UserFilter>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };
    let user = token.claims.user;
    let assessments = match user.get_assessments(&pool, Some(filter.into_inner()), Some(subject_filter.into_inner()), Some(person_filter.into_inner())).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(assessments)
}
