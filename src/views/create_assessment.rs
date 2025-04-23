use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use sqlx::QueryBuilder;
use sqlx::MySql;
use futures::future::join_all;

use crate::jwt::validate;
use crate::user::Role;

#[derive(Debug, sqlx::Type, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[sqlx(type_name = "ENUM('exam','homework','project','oral','remedial','selfassessable')")]
#[serde(rename_all = "lowercase")]
pub enum AssessmentType {
    Exam,
    Homework,
    Project,
    Oral,
    Remedial,
    Selfassessable,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewTask {
    subject: u64,
    task: String,
    due_date: String,
    #[serde(rename = "type")]
    type_: AssessmentType,
}


#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewSelfassessable{
    questions: Vec<String>,
    correct: Vec<String>,
    incorrect1: Vec<String>,
    incorrect2: Option<Vec<String>>,
    incorrect3: Option<Vec<String>>,
    incorrect4: Option<Vec<String>>,
}

impl NewSelfassessable{
    pub fn validate(&self) -> bool {
        if self.correct.len() != self.incorrect1.len(){
            return false;
        }
        if self.correct.len() != self.questions.len(){
            return false;
        }
        if let Some(v) = &self.incorrect2{
            if self.correct.len() != v.len() {
                return false;
            }
        }
        if let Some(v) = &self.incorrect3{
            if self.correct.len() != v.len() {
                return false;
            }
        }
        if let Some(v) = &self.incorrect4{
            if self.correct.len() != v.len() {
                return false;
            }
        }
        true
    }
   
    pub fn generate_query(&self,assessable_id: u64) -> Vec<QueryBuilder<MySql>> {
        let mut queries = vec![];

        let count = self.correct.len();

        for i in 0..count {
            let mut query: QueryBuilder<MySql> = QueryBuilder::new(
                "INSERT INTO selfassessable_tasks (selfassessable_id, question, correct, incorrect1"
            );

            if self.incorrect2.as_ref().map_or(false, |v| v.len() > i) {
                query.push(", incorrect2");
            }
            if self.incorrect3.as_ref().map_or(false, |v| v.len() > i) {
                query.push(", incorrect3");
            }
            if self.incorrect4.as_ref().map_or(false, |v| v.len() > i) {
                query.push(", incorrect4");
            }

            query.push(") VALUES (");
            query.push_bind(assessable_id);
            query.push(", ");
            query.push_bind(&self.questions[i]);
            query.push(", ");
            query.push_bind(&self.correct[i]);
            query.push(", ");
            query.push_bind(&self.incorrect1[i]);

            if let Some(ref vals) = self.incorrect2 {
                if let Some(val) = vals.get(i) {
                    query.push(", ").push_bind(val);
                }
            }
            if let Some(ref vals) = self.incorrect3 {
                if let Some(val) = vals.get(i) {
                    query.push(", ").push_bind(val);
                }
            }
            if let Some(ref vals) = self.incorrect4 {
                if let Some(val) = vals.get(i) {
                    query.push(", ").push_bind(val);
                }
            }

            query.push(")");

            queries.push(query);
        }

        queries
    }

}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Payload {
    newtask: NewTask,
    newselfassessable: Option<NewSelfassessable>
}


#[post("/api/v1/create_assessment/")]
pub async fn create_assessment(
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

    let user_id = token.claims.subject as u64;

    let role = token.claims.role;

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
