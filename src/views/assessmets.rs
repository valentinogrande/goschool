use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use sqlx::mysql::MySqlPool;

use crate::filters::{AssessmentFilter, SubjectFilter, UserFilter};
use crate::jwt::validate;
use crate::structs::{AssessmentType, AssessmentWithSelfassessableId, Payload};
use crate::traits::{Get, Post};

#[get("/api/v1/assessments/")]
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
    let assessments = match user
        .get_assessments(
            &pool,
            filter.into_inner(),
            subject_filter.into_inner(),
            person_filter.into_inner(),
        )
        .await
    {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let mut response = Vec::new();
    for assessment in assessments {
        let selfassessable_id = if assessment.type_ == AssessmentType::Selfassessable {
            let id: Option<u64> =
                sqlx::query_scalar("SELECT id FROM selfassessables WHERE assessment_id = ?")
                    .bind(assessment.id)
                    .fetch_optional(pool.get_ref())
                    .await
                    .unwrap_or(None);
            id
        } else {
            None
        };
        response.push(AssessmentWithSelfassessableId {
            id: assessment.id,
            subject_id: assessment.subject_id,
            task: assessment.task,
            due_date: assessment.due_date,
            created_at: assessment.created_at,
            type_: assessment.type_,
            selfassessable_id,
        });
    }
    HttpResponse::Ok().json(response)
}

#[post("/api/v1/assessments/")]
pub async fn post_assessment(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<Payload>,
) -> impl Responder {
    let jwt = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(jwt.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = token.claims.user;

    user.post_assessment(&pool, payload.into_inner()).await
}
