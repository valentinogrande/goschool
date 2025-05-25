use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post};
use sqlx::mysql::MySqlPool;

use crate::jwt::validate;
use crate::structs::{Role, Payload, AssessmentType};
use crate::filters::{AssessmentFilter, SubjectFilter, UserFilter};
use futures::future::join_all;

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
    let assessments = match user.get_assessments(&pool, Some(filter.into_inner()), Some(subject_filter.into_inner()), Some(person_filter.into_inner())).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    HttpResponse::Ok().json(assessments)
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

    let user_id = token.claims.user.id;

    let role = token.claims.user.role;

    if role == Role::teacher {
        let teacher_subject: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM subjects WHERE teacher_id = ? AND id = ?)")
            .bind(user_id)
            .bind(payload.newtask.subject)
            .fetch_one(pool.get_ref())
            .await {
            Ok(s) => s,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };
        if !teacher_subject {
            return HttpResponse::Unauthorized().finish();
        }
    }
    else if role == Role::admin{}
    else {
        return HttpResponse::BadRequest().finish();
    }


    if payload.newtask.type_ == AssessmentType::Selfassessable{
        
        let selfassessable = match &payload.newselfassessable {
            Some(a) => a,
            None => return HttpResponse::BadRequest().finish(),
        };

        if !(selfassessable.validate()){
            return HttpResponse::BadRequest().finish();
        }

        let insert_result = match sqlx::query("INSERT INTO assessments (task, subject_id, type, due_date) VALUES (?, ?, ?, ?)")
        .bind(&payload.newtask.task)
        .bind(payload.newtask.subject)
        .bind(&payload.newtask.type_)
        .bind(&payload.newtask.due_date)
        .execute(pool.get_ref())
        .await
    {
        Ok(res) => res,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
        let assessment_id = insert_result.last_insert_id();
    
        let assessable = match sqlx::query("INSERT INTO selfassessables (assessment_id) VALUES (?)").bind(assessment_id).execute(pool.get_ref()).await {
            Ok(r)=>r,
            Err(_)=>return HttpResponse::InternalServerError().finish(),
        };
        let assessable_id = assessable.last_insert_id();
        let mut queries = selfassessable.generate_query(assessable_id);

        let results = join_all(
            queries.iter_mut().map(|q| {
                q.build().execute(&**pool)  
            })
        ).await;
        for res in results {
            match res {
                Ok(_) => {},
                Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
            }
        }

        return HttpResponse::Created().finish()
    }
    else{

        let insert_result = sqlx::query("INSERT INTO assessments (task, subject_id, type, due_date) VALUES (?, ?, ?, ?)")
        .bind(&payload.newtask.task)
        .bind(payload.newtask.subject)
        .bind(&payload.newtask.type_)
        .bind(&payload.newtask.due_date)
        .execute(pool.get_ref())
        .await;

        match insert_result {
           Ok(_) => HttpResponse::Created().finish(),
           Err(_) => HttpResponse::BadRequest().finish(),
        }
    
    }
}
