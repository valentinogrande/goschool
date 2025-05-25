use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::structs::{NewGrade, Role};
use crate::filters::GradeFilter;


#[get("/api/v1/grades/")]
pub async fn get_grades(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    filter: web::Query<GradeFilter>,
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

    let grades = match user.get_grades(pool, Some(filter.into_inner())).await { 
        Ok(g) => g,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(grades)
}

#[post("/api/v1/grades/")]
pub async fn post_grade(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    grade: web::Json<NewGrade>,
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
    
    if user.role != Role::teacher && user.role != Role::admin {
            return HttpResponse::Unauthorized().finish();
        }

    let teacher_subject: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM subjects WHERE teacher_id = ? AND id = ?)")
        .bind(user.id)
        .bind(grade.subject)
        .fetch_one(pool.get_ref())
        .await {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if !teacher_subject {
        return HttpResponse::Unauthorized().finish();
    }

    let course = match sqlx::query_scalar::<_, u64>("SELECT course_id FROM subjects WHERE id = ?")
    .bind(grade.subject)
        .fetch_one(pool.get_ref())
        .await{
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let student_course: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND course_id = ?)")
        .bind(grade.student_id)
        .bind(course)
        .fetch_one(pool.get_ref())
        .await {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if !student_course{
        return HttpResponse::Unauthorized().finish();
    }
    if let Some(assessment_id) = grade.assessment_id{
    
        let assessment_verify: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assessments WHERE id = ? AND subject_id = ?)")
            .bind(assessment_id)
            .bind(grade.subject)
            .fetch_one(pool.get_ref())
            .await{
            Ok(s) => s,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };
        if !assessment_verify{
            return HttpResponse::Unauthorized().finish();
        }
        let assessment_already_exixts: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM grades WHERE assessment_id = ? AND student_id = ? )")
        .bind(assessment_id)
        .bind(grade.student_id)
        .fetch_one(pool.get_ref())
        .await {
            Ok(s) => s,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };
        if assessment_already_exixts{
            return HttpResponse::Unauthorized().finish();
        }
        let result = sqlx::query("INSERT INTO grades (assessment_id, student_id, grade_type, description, grade, subject_id) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(assessment_id)
            .bind(grade.student_id)
            .bind(&grade.grade_type)
            .bind(&grade.description)
            .bind(grade.grade)
            .bind(grade.subject)
            .execute(pool.get_ref())
            .await;
        if result.is_err() {
            return HttpResponse::InternalServerError().finish();
        }
        else {
            return HttpResponse::Created().finish();
        }
    }
     let result = sqlx::query("INSERT INTO grades (student_id, grade_type, description, grade, subject_id) VALUES (?, ?, ?, ?, ?)")
        .bind(grade.student_id)
        .bind(&grade.grade_type)
        .bind(&grade.description)
        .bind(grade.grade)
        .bind(grade.subject)
        .execute(pool.get_ref())
        .await;
    if result.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    else {
        return HttpResponse::Created().finish();
    }
}
