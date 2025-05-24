use serde::{Serialize, Deserialize};
use sqlx::{FromRow, QueryBuilder, MySql, Type};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc, NaiveDate};


#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewMessage {
    pub courses: String,
    pub title: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct PhotoUrlResponse {
    pub url: String,
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, sqlx::Decode)]
pub struct PersonalData {
    pub full_name: String,
    pub phone_number: String,
    pub address: String,
    pub birth_date: NaiveDate,  
}


#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, sqlx::Decode)]
pub struct Subject {
    pub id: u64,
    pub name: String,
    pub teacher_id: u64,
    pub course_id: u64,
}


#[derive(Debug, Type, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GradeType {
    Numerical,
    Conceptual,
    Percentage,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Grade {
    pub id: u64,
    pub description: Option<String>,
    pub grade: Decimal,
    pub student_id: u64,
    pub subject_id: u64,
    pub assessment_id: Option<u64>,
    pub grade_type: Option<GradeType>,
    pub created_at: Option<DateTime<Utc>>,
}


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "ENUM('admin', 'teacher', 'student', 'preceptor', 'father')")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    admin,
    teacher,
    student,
    preceptor,
    father,
}

#[derive(Deserialize, Serialize)]
pub struct NewGrade {
    pub subject: u64,
    pub assessment_id: Option<u64>,
    pub student_id: u64,
    pub grade_type: GradeType,
    pub description: String,
    pub grade: f32,
}

#[derive(Debug, Type, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Deserialize, Serialize)]
pub struct NewTask {
    pub subject: u64,
    pub task: String,
    pub due_date: String,
    #[serde(rename = "type")]
    pub type_: AssessmentType,
}


#[derive(Deserialize, Serialize)]
pub struct NewSelfassessable{
    pub questions: Vec<String>,
    pub correct: Vec<String>,
    pub incorrect1: Vec<String>,
    pub incorrect2: Option<Vec<String>>,
    pub incorrect3: Option<Vec<String>>,
    pub incorrect4: Option<Vec<String>>,
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

#[derive(Deserialize, Serialize)]
pub struct Payload {
    pub newtask: NewTask,
    pub newselfassessable: Option<NewSelfassessable>
}


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Assessment {
    pub id: u64,
    pub subject_id: u64,
    pub task: String,
    pub due_date: NaiveDate,  
    pub created_at: DateTime<Utc>,
    #[sqlx(rename = "type")] 
    #[serde(rename = "type")]
    pub type_: AssessmentType,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub sender_id: u64,
    pub created_at: Option<DateTime<Utc>>,
}

