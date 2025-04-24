use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::user::Role;
use crate::jwt::validate;


#[derive(Debug, sqlx::Type, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NewSubmissionSelfAssessable{
    assessment_id: u64,
    answers: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, FromRow)]
pub struct Selfassessable {
    pub correct: String,
    pub incorrect1: String,
    pub incorrect2: Option<String>,
    pub incorrect3: Option<String>,
    pub incorrect4: Option<String>,
}



impl NewSubmissionSelfAssessable {
    pub async fn get_answers(&self, pool: &MySqlPool) -> Result<String, sqlx::Error> {
        let tasks: Vec<Selfassessable> = sqlx::query_as::<_, Selfassessable>(
                r#"
                SELECT correct, incorrect1, incorrect2, incorrect3, incorrect4
                  FROM selfassessable_tasks st
                  JOIN selfassessables s ON s.id = st.selfassessable_id
                 WHERE s.assessment_id = ?
                "#)
            .bind(self.assessment_id)
            .fetch_all(pool)
            .await?;

        log::info!("loaded tasks: {:#?}", tasks);

        let mut indices = Vec::with_capacity(self.answers.len());

        for (task, submitted) in tasks.iter().zip(&self.answers) {
            let idx = if &task.correct == submitted {
                1
            } else if &task.incorrect1 == submitted {
                2
            } else if task.incorrect2.as_deref() == Some(submitted.as_str()) {
                3
            } else if task.incorrect3.as_deref() == Some(submitted.as_str()) {
                4
            } else if task.incorrect4.as_deref() == Some(submitted.as_str()) {
                5
            } else {
                // You can choose to error here instead of pushing 0
                0
            };
            indices.push(idx);
        }

        log::info!("mapped answer indices: {:?}", indices);

        let result = indices
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");

        Ok(result)
    }
}


#[post("/api/v1/create_selfassessable_submission/")]
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

    let user_id = token.claims.subject as u64;

    if token.claims.role != Role::student {
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
