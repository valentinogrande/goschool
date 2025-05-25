use serde::{Serialize, Deserialize};
use sqlx::{FromRow, QueryBuilder, MySql, Type, MySqlPool, Decode};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc, NaiveDate};

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

#[derive(Debug, FromRow, Serialize)]
pub struct Course {
    pub id: u64,
    pub year: i32,
    pub division: String,
    pub level: Level,
    pub shift: Shift,
    pub preceptor_id: Option<u64>,
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
pub struct NewMessage {
    pub courses: String,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Type, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NewSubmissionSelfAssessable {
    assessment_id: u64,
    answers: Vec<String>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct NewTask {
    pub subject: u64,
    pub task: String,
    pub due_date: String,
    #[serde(rename = "type")]
    pub type_: AssessmentType,
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

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct Selfassessable {
    pub correct: String,
    pub incorrect1: String,
    pub incorrect2: Option<String>,
    pub incorrect3: Option<String>,
    pub incorrect4: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, Decode)]
pub struct Subject {
    pub id: u64,
    pub name: String,
    pub teacher_id: u64,
    pub course_id: u64,
}

#[derive(Debug, Type, Serialize)]
#[sqlx(type_name = "enum", rename_all = "lowercase")]
pub enum Level {
    Primary,
    Secondary,
}

#[derive(Debug, Type, Serialize)]
#[sqlx(type_name = "enum", rename_all = "lowercase")]
pub enum Shift {
    Morning,
    Afternoon,
}

impl NewSubmissionSelfAssessable {
    pub async fn get_answers(&self, pool: &MySqlPool) -> Result<String, sqlx::Error> {
        let tasks: Vec<Selfassessable> = sqlx::query_as::<_, Selfassessable>(
            r#"
            SELECT correct, incorrect1, incorrect2, incorrect3, incorrect4
              FROM selfassessable_tasks st
              JOIN selfassessables s ON s.id = st.selfassessable_id
             WHERE s.assessment_id = ?
            "#,
        )
        .bind(self.assessment_id)
        .fetch_all(pool)
        .await?;

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
                0
            };
            indices.push(idx);
        }

        let result = indices
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");

        Ok(result)
    }
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

    pub fn generate_query(&self, assessable_id: u64) -> Vec<QueryBuilder<MySql>> {
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

