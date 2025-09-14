use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, FromRow, MySql, QueryBuilder, Type};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MySelf {
    pub role: Role,
    pub id: u64,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: u64,
    pub password: String,
    pub email: String,
    pub role: Role,
    pub last_login: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewUser {
    pub password: String,
    pub email: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct CredentialsRole {
    pub email: String,
    pub password: String,
    pub role: Role,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Decode, sqlx::Type, FromRow)]
pub struct PubUser{
    pub id: u64,
    pub photo: String,
    pub course_id: u64,
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

#[derive(Debug, FromRow, Serialize, Deserialize)]
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

#[derive(sqlx::Type, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[sqlx(type_name = "ENUM('exam','homework','project','oral','remedial','selfassessable')")]
#[serde(rename_all = "lowercase")]
pub enum AssessmentType {
    #[sqlx(rename = "exam")]
    Exam,
    #[sqlx(rename = "homework")]
    Homework,
    #[sqlx(rename = "project")]
    Project,
    #[sqlx(rename = "oral")]
    Oral,
    #[sqlx(rename = "remedial")]
    Remedial,
    #[sqlx(rename = "selfassessable")]
    Selfassessable,
}

#[derive(Debug, FromRow, Serialize)]
pub struct PendingSelfassessableGrade {
    pub id: u64,
    pub selfassessable_id: u64,
    pub student_id: u64,
    pub grade: Decimal,
}

#[derive(Debug, FromRow, Serialize, sqlx::Type)]
pub struct Course {
    pub id: u64,
    pub year: i32,
    pub division: String,
    pub level: String,
    pub shift: String,
    pub preceptor_id: Option<u64>,
    pub name: String
}


#[derive(Debug, FromRow, Serialize, Deserialize, sqlx::Type)]
pub struct Assistance {
    pub id: u64,
    pub student_id: u64,
    pub presence: String,
    pub date: NaiveDate,

}

#[derive(Debug, FromRow, Serialize, Deserialize, sqlx::Type)]
pub struct NewAssistance {
    pub student_id: u64,
    pub presence: String,
    pub date: NaiveDate,
}

#[derive(Debug, FromRow, Serialize, Deserialize, sqlx::Type)]
pub struct UpdateAssistance {
    pub student_id: u64,
    pub presence: String,
    pub date: NaiveDate,
}

#[derive(Debug, FromRow, Serialize, Deserialize, sqlx::Type)]
pub struct DisciplinarySanction {
    pub id: u64,
    pub student_id: u64,
    pub sanction_type: String,
    pub quantity: i32,
    pub description: String,
    pub date: NaiveDate,
}

#[derive(Debug, FromRow, Serialize, Deserialize, sqlx::Type)]
pub struct NewDisciplinarySanction {
    pub student_id: u64,
    pub sanction_type: String,
    pub quantity: i32,
    pub description: String,
    pub date: NaiveDate,
}

#[derive(Debug, FromRow, Serialize, Deserialize, sqlx::Type)]
pub struct UpdateDisciplinarySanction {
    pub student_id: u64,
    pub sanction_type: String,
    pub quantity: i32,
    pub description: String,
    pub date: NaiveDate,
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


#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Timetable {
    pub id: u64, 
    pub course_id: u64,
    pub subject_id: u64,
    pub day: String,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime
}

#[derive(Debug, Type, Serialize, Deserialize)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GradeType {
    Numerical,
    Conceptual,
    Percentage,
}

#[derive(Serialize, Deserialize)]
pub struct NewGrade {
    pub subject: u64,
    pub assessment_id: Option<u64>,
    pub student_id: u64,
    pub grade_type: GradeType,
    pub description: String,
    pub grade: f32,
}

#[derive(Serialize, Deserialize)]
pub struct NewSubjectMessage {
    pub sender_id: u64,
    pub subject_id: u64,
    pub title: String,
    pub content: String,
    #[serde(rename = "type")]
    pub type_: SubjectMessageType,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct SubjectMessage {
    pub id: u64,
    pub sender_id: u64,
    pub subject_id: u64,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub type_: SubjectMessageType,
}

#[derive(sqlx::Type, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[sqlx(type_name = "ENUM('message','link','file')")]
#[serde(rename_all = "lowercase")]
pub enum SubjectMessageType {
    #[sqlx(rename = "message")]
    Message,
    #[sqlx(rename = "link")]
    Link,
    #[sqlx(rename = "file")]
    File,
}

#[derive(Serialize, Deserialize)]
pub struct NewMessage {
    pub courses: String,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Type, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NewSubmissionSelfAssessable {
    pub assessment_id: u64,
    pub answers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewSelfassessable {
    pub questions: Vec<String>,
    pub correct: Vec<String>,
    pub incorrect1: Vec<String>,
    pub incorrect2: Option<Vec<String>>,
    pub incorrect3: Option<Vec<String>>,
    pub incorrect4: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct Selfassessable {
    pub id: u64,
    pub question: String,
    pub correct: String,
    pub incorrect1: String,
    pub incorrect2: Option<String>,
    pub incorrect3: Option<String>,
    pub incorrect4: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct PublicSelfassessable {
    pub id: u64,
    pub question: String,
    pub op1: String,
    pub op2: String,
    pub op3: Option<String>, 
    pub op4: Option<String>,
    pub op5: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct SelfassessableResponse {
    pub id: u64,
    pub selfassessable_id: u64,
    pub answers: String,
    pub student_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewTask {
    pub subject: u64,
    pub task: String,
    pub due_date: String,
    #[serde(rename = "type")]
    pub type_: AssessmentType,
}

#[derive(Serialize, Deserialize, FromRow, Decode)]
pub struct PublicPersonalData {
    pub full_name: String,
    pub photo: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, Decode)]
pub struct PersonalData {
    pub full_name: String,
    pub phone_number: String,
    pub address: String,
    pub birth_date: NaiveDate,
}

#[derive(Serialize)]
pub struct PhotoUrlResponse {
    pub url: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub sender_id: u64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub newtask: NewTask,
    pub newselfassessable: Option<NewSelfassessable>,
}

#[derive(Serialize, Deserialize, FromRow, Decode, Debug)]
pub struct Subject {
    pub id: u64,
    pub name: String,
    pub teacher_id: u64,
    pub course_id: u64,
    pub course_name: String,
}

impl NewSelfassessable {
    pub fn validate(&self) -> bool {
        if self.correct.len() != self.incorrect1.len() {
            return false;
        }
        if self.correct.len() != self.questions.len() {
            return false;
        }
        if let Some(v) = &self.incorrect2 {
            if self.correct.len() != v.len() {
                return false;
            }
        }
        if let Some(v) = &self.incorrect3 {
            if self.correct.len() != v.len() {
                return false;
            }
        }
        if let Some(v) = &self.incorrect4 {
            if self.correct.len() != v.len() {
                return false;
            }
        }
        true
    }

    pub fn generate_query(&self, assessable_id: u64) -> Vec<QueryBuilder<'_, MySql>> {
        let mut queries = vec![];
        let count = self.correct.len();

        for i in 0..count {
            let mut query: QueryBuilder<MySql> = QueryBuilder::new(
                "INSERT INTO selfassessable_tasks (selfassessable_id, question, correct, incorrect1",
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

#[derive(Serialize, Deserialize)]
pub struct UpdateUser {
    pub password: Option<String>,
    pub email: Option<String>,
    pub role: Option<Role>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCourse {
    pub year: Option<i32>,
    pub division: Option<String>,
    pub level: Option<String>,
    pub shift: Option<String>,
    pub preceptor_id: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGrade {
    pub description: Option<String>,
    pub grade: Option<Decimal>,
    pub student_id: Option<u64>,
    pub subject_id: Option<u64>,
    pub assessment_id: Option<u64>,
    pub grade_type: Option<GradeType>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateSubject {
    pub name: Option<String>,
    pub teacher_id: Option<u64>,
    pub course_id: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateAssessment {
    pub subject_id: Option<u64>,
    pub task: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub type_: Option<AssessmentType>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateMessage {
    pub title: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateTimetable {
    pub course_id: Option<u64>,
    pub subject_id: Option<u64>,
    pub day: Option<String>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePersonalData {
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub address: Option<String>,
    pub birth_date: Option<NaiveDate>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateSubjectMessage {
    pub title: Option<String>,
    pub content: Option<String>,
    pub type_: Option<SubjectMessageType>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateSubmission {
    pub path: Option<String>,
    pub student_id: Option<u64>,
    pub task_id: Option<u64>,
}
