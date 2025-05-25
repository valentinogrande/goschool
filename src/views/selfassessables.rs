use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post};
use sqlx::mysql::MySqlPool;
use sqlx::QueryBuilder;


use crate::jwt::validate;
use crate::structs::{Role, Grade, NewSubmissionSelfAssessable};
use crate::filters::{GradeFilter, SubjectFilter};

#[get("/api/v1/get_grade_selfassessable/{selfassessable_id}/")]
pub async fn get_grade_selfassessable(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    selfassessable_id: web::Path<u64>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().json("Missing JWT cookie"),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid JWT token"),
    };

    let token_id = token.claims.subject as u64;
   
    let role = token.claims.role;
   
    todo!();
    if role == Role::father{

    }


    let mut query = QueryBuilder::new("SELECT * FROM grades WHERE student_id = ");
    query.push_bind(student_id);

    if let Some(subject_id) = filter.subject_id {
        query.push(" AND subject_id = ");
        query.push_bind(subject_id);
    }

    if let Some(ref description) = filter.description {
        query.push(" AND description LIKE ");
        query.push_bind(format!("%{}%", description));
    }
    

    let grades: Vec<Grade> = match query
        .build_query_as()
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().json(grades)
}

#[post("/api/v1/selfassessables/")]
pub async fn create_selfassessable_submission(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    selfassessablesubmission: web::Json<NewSubmissionSelfAssessable>,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.user.id;

    if token.claims.user.role != Role::student {
        return HttpResponse::Unauthorized().finish();
    }

    let user_course = match sqlx::query_scalar::<_, u64>(
        "SELECT course_id FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await {
        Ok(course_id) => course_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let (assessment_type, subject_id): (String, u64) = match sqlx::query_as(
        "SELECT type, subject_id FROM assessments WHERE id = ?"
    )
    .bind(selfassessablesubmission.assessment_id)
    .fetch_one(pool.get_ref())
    .await {
        Ok(res) => res,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if assessment_type != "selfassessable" {
        return HttpResponse::BadRequest().body("submission are only valid for selfassables");
    }

    let assessable_course = match sqlx::query_scalar::<_, u64>(
        "SELECT course_id FROM subjects WHERE id = ?"
    )
    .bind(subject_id)
    .fetch_one(pool.get_ref())
    .await {
        Ok(course_id) => course_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if assessable_course != user_course {
        return HttpResponse::Unauthorized().finish();
    }

    let selfassessable_id = match sqlx::query_scalar::<_, u64>(
        "SELECT id FROM selfassessables WHERE assessment_id = ?"
    )
    .bind(selfassessablesubmission.assessment_id)
    .fetch_one(pool.get_ref())
    .await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let already_exists = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM selfassessable_submissions WHERE student_id = ? AND selfassessable_id = ?)"
    )
    .bind(user_id)
    .bind(selfassessable_id)
    .fetch_one(pool.get_ref())
    .await {
        Ok(exists) => exists,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if already_exists {
        return HttpResponse::BadRequest().body("You already submitted this homework");
    }
    
    let answers = match selfassessablesubmission.get_answers(&pool).await{
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    
    log::info!("{:?}",answers);
    match sqlx::query(
        "INSERT INTO selfassessable_submissions (selfassessable_id, student_id, answers) VALUES (?, ?, ?)"
    )
    .bind(selfassessable_id)
    .bind(user_id)
    .bind(answers)
    .execute(pool.get_ref())
    .await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
