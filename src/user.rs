use sqlx::{MySqlPool, QueryBuilder};
use actix_web::web;
use chrono::Utc;
use std::env;

use crate::filters::{GradeFilter, UserFilter, SubjectFilter, AssessmentFilter, MessageFilter};
use crate::structs::{Assessment, Grade, Role, Subject, PersonalData, Message, Course};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MySelf{
    pub role: Role,
    pub id: u64
}

impl MySelf{
    pub fn new(id: u64, role: Role) -> Self{
        Self { role, id }
    }
   pub async fn get_students(
        &self,
        pool: web::Data<MySqlPool>,
        filter: Option<UserFilter>,
    ) -> Result<Vec<u64>, sqlx::Error> {
        let mut query = sqlx::QueryBuilder::new("SELECT DISTINCT users.id FROM users ");

        match &self.role {
            Role::teacher => {
                query.push("JOIN courses c ON users.course_id = c.id ");
                query.push("JOIN subjects s ON s.course_id = c.id ");
                query.push("WHERE s.teacher_id = ?");
                query.push_bind(self.id);
            }
Role::student => {
                return Ok(vec![self.id]);
            }
            Role::preceptor => {
                query.push("JOIN courses c ON users.course_id = c.id ");
                query.push("WHERE c.preceptor_id = ?");
                query.push_bind(self.id);
            }
            Role::father => {
                query.push("JOIN families f ON f.student_id = users.id ");
                query.push("WHERE f.father_id = ?");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
        }
        if let Some(f) = filter {
            if let Some(c) = f.course {
                query.push(" AND users.course_id = ?");
                query.push_bind(c);
            }

            if let Some(n) = f.name {
                query.push(" AND EXISTS (SELECT 1 FROM personal_data pd WHERE pd.user_id = users.id AND pd.full_name LIKE ?)");
                query.push_bind(format!("%{}%", n));
            }
        }
        let res = query
            .build_query_scalar::<u64>()
            .fetch_all(pool.as_ref())
            .await;

        res
    }
    
    pub async fn get_courses(&self, pool: &MySqlPool) -> Result<Vec<Course>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT * FROM courses c ");
        match self.role {
            Role::student => {
                query.push("JOIN users u ON c.id = u.course_id WHERE u.id = ?");
                query.push_bind(self.id);
            },
            Role::admin => {
                query.push("WHERE 1=1");
            },
            Role::preceptor => {
                query.push("WHERE preceptor_id = ?");
                query.push_bind(self.id);
            },
            Role::father => {
                query.push("JOIN users u ON c.id = u.course_id JOIN families f ON f.student_id = u.id WHERE f.father_id = ?");
                query.push_bind(self.id);
            },
            Role::teacher => {
                query.push("JOIN subjects s ON c.id = s.course_id WHERE s.teacher_id = ?");
                query.push_bind(self.id);
            },
        };

        let res = query
            .build_query_as()
            .fetch_all(pool)
            .await;
        res
    }
    pub async fn get_grades(&self, pool: web::Data<MySqlPool>, filter: 
    Option<GradeFilter>) -> Result<Vec<Grade>, sqlx::Error>{
        let mut query = QueryBuilder::new("SELECT * FROM grades ");
        match self.role {
            Role::student => {
                query.push("WHERE student_id =");
                query.push_bind(self.id);
            },
            Role::teacher => {
                query.push("SELECT * FROM grades g JOIN subjects s ON g.subject_id = s.id WHERE s.teacher_id =");
                query.push_bind(self.id);
            }
            Role::admin => {
            query.push("WHERE 1=1");
            }
            Role::father => {
                query.push("SELECT * FROM grades g JOIN families f ON g.student_id = f.student_id WHERE f.father_id =");
                query.push_bind(self.id);
            }
            Role::preceptor => {
                query.push("SELECT * FROM grades g JOIN subjects s ON g.subject_id = s.id JOIN courses c ON s.course_id = c.id WHERE c.preceptor_id =");
                query.push_bind(self.id);
            }
        };
        if let Some(f) = filter {
            if let Some(c) = f.student_id {
                query.push(" AND student_id = ?");
                query.push_bind(c);
            }
            if let Some(s) = f.subject_id {
                query.push(" AND subject_id = ?");
                query.push_bind(s);
            }
            if let Some(d) = f.description {
                query.push(" AND description = ?");
                query.push_bind(d);
            }
        }
        let res = query
            .build_query_as()
            .fetch_all(pool.as_ref())
            .await;
        res
    }
    pub async fn get_subjects(&self, pool: &MySqlPool, filter: Option<SubjectFilter>) -> Result<Vec<Subject>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT * FROM subjects s ");
        match self.role {
            Role::teacher => {
                query.push("WHERE teacher_id =");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
            Role::student => {
                query.push(
                "JOIN courses c ON s.course_id = c.id \
                 JOIN users u ON c.id = u.course_id \
                 WHERE u.id = "
                );
                query.push_bind(self.id);
            }
            Role::preceptor => {
                query.push("JOIN courses c ON s.course_id = c.id WHERE c.preceptor_id =");
                query.push_bind(self.id);
            }
            Role::father => {
                query.push(
                "JOIN users u ON s.course_id = u.course_id \
                 JOIN families f ON f.student_id = u.id \
                 WHERE f.father_id = "
                );
                query.push_bind(self.id);
            }
        };
        if let Some(f) = filter {
            if let Some(c) = f.course_id {
                query.push(" AND s.course_id = ?");
                query.push_bind(c);
            }
            if let Some(n) = f.name {
                query.push(" AND s.name LIKE ?");
                query.push_bind(n);
            }
            if let Some(t) = f.teacher_id {
                query.push(" AND s.teacher_id = ?");
                query.push_bind(t);
            }
        }
        let res = query
            .build_query_as()
            .fetch_all(pool)
            .await;
        res
    }
   
    pub async fn get_assessments(
        &self,
        pool: &MySqlPool,
        filter: Option<AssessmentFilter>,
        subject_filter: Option<SubjectFilter>,
        person_filter: Option<UserFilter>,
    ) -> Result<Vec<Assessment>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT a.* FROM assessments a JOIN subjects s ON a.subject_id = s.id ");

        match self.role {
            Role::teacher => {
                query.push("WHERE s.teacher_id = ");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
            Role::father => {
                let subjects: Vec<u64> = sqlx::query_scalar(
                    "SELECT s.id FROM subjects s
                     JOIN users u ON s.course_id = u.course_id
                     JOIN families f ON f.student_id = u.id
                     WHERE f.father_id = ?"
                )
                .bind(self.id)
                .fetch_all(pool)
                .await?;

                if subjects.is_empty() {
                    return Ok(Vec::new());
                }

                query.push("WHERE a.subject_id IN (");
                let mut separated = query.separated(", ");
                for id in subjects.iter() {
                    separated.push_bind(*id);
                }
                query.push(")");
            }
            Role::student => {
                let subjects: Vec<u64> = sqlx::query_scalar(
                    "SELECT s.id FROM subjects s
                     JOIN users u ON s.course_id = u.course_id
                     WHERE u.id = ?"
                )
                .bind(self.id)
                .fetch_all(pool)
                .await?;

                if subjects.is_empty() {
                    return Ok(Vec::new());
                }

                query.push("WHERE a.subject_id IN (");
                let mut separated = query.separated(", ");
                for id in subjects.iter() {
                    separated.push_bind(*id);
                }
                query.push(")");
            }
            Role::preceptor => {
                let subjects: Vec<u64> = sqlx::query_scalar(
                    "SELECT s.id FROM subjects s
                     JOIN courses c ON s.course_id = c.id
                     WHERE c.preceptor_id = ?"
                )
                .bind(self.id)
                .fetch_all(pool)
                .await?;

                if subjects.is_empty() {
                    return Ok(Vec::new());
                }

                query.push("WHERE a.subject_id IN (");
                let mut separated = query.separated(", ");
                for id in subjects.iter() {
                    separated.push_bind(*id);
                }
                query.push(")");
            }
        }

        // Subject filters
        if let Some(f) = subject_filter {
            if let Some(c) = f.course_id {
                query.push(" AND s.course_id = ");
                query.push_bind(c);
            }
            if let Some(n) = f.name {
                query.push(" AND s.name LIKE ");
                query.push_bind(format!("%{}%", n));
            }
            if let Some(t) = f.teacher_id {
                query.push(" AND s.teacher_id = ");
                query.push_bind(t);
            }
        }

        // Person filters
        if let Some(f) = person_filter {
            if let Some(n) = f.name {
                query.push(
                    " AND EXISTS (
                        SELECT 1 FROM personal_data pd
                        WHERE pd.user_id = a.user_id
                        AND pd.full_name LIKE "
                );
                query.push_bind(format!("%{}%", n));
                query.push(")");
            }
            if let Some(c) = f.course {
                query.push(
                    " AND EXISTS (
                        SELECT 1 FROM users u
                        WHERE u.id = a.user_id
                        AND u.course_id = "
                );
                query.push_bind(c);
                query.push(")");
            }
            if let Some(i) = f.id {
                query.push(" AND a.user_id = ");
                query.push_bind(i);
            }
        }
        if let Some(f) = filter {
            if let Some(due) = f.due {
                if due {
                    let actual_date = Utc::now();
                    query.push(" AND a.due_date >= ?");
                    query.push_bind(actual_date);
                }
            }
            if let Some(t) = f.task {
                query.push(" AND a.task LIKE ");
                query.push_bind(format!("%{}%", t));
            }

            
            
        }

        let res = query
            .build_query_as::<Assessment>()
            .fetch_all(pool)
            .await;

        res
    }
    pub async fn get_personal_data(&self, pool: &MySqlPool) -> Result<PersonalData, sqlx::Error> {
        let res = sqlx::query_as("SELECT * FROM personal_data WHERE user_id = ?")
            .bind(self.id)
            .fetch_one(pool)
            .await;
        res
    }
    pub async fn get_profile_picture(&self, pool: &MySqlPool) -> Result<String, sqlx::Error> {
        let photo_filename: String = match sqlx::query_scalar("SELECT photo FROM users WHERE id = ?")
        .bind(self.id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(path)) => path,
        Ok(None) => "default.jpg".to_string(),
        Err(e) => return Err(e),
    };

        let base_url = env::var("BASE_URL").expect("BASE_URL must be set");
        let url = format!("{}/uploads/profile_pictures/{}", base_url, photo_filename);
        Ok(url)
    }
    pub async fn get_messages(&self, pool: &MySqlPool, filter: Option<MessageFilter>) -> Result<Vec<Message>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT * FROM messages m JOIN message_courses mc ON mc.message_id = m.id ");
        match self.role {
            Role::student => {
                query.push("JOIN users u ON u.course_id = mc.course_id WHERE u.id = ?");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
            Role::father => {
                query.push("JOIN users u ON u.course_id = mc.course_id JOIN families f ON f.student_id = u.id WHERE f.father_id = ?");
                query.push_bind(self.id);
            }
            Role::teacher => {
                query.push("JOIN subjects s ON mc.course_id = s.course_id WHERE s.teacher_id = ?");
                query.push_bind(self.id);
            }
            Role::preceptor => {
                query.push("JOIN courses c ON mc.course_id = c.id WHERE c.preceptor_id = ?");
                query.push_bind(self.id);
            }
        };
        if let Some(f) = filter {
            if let Some(c) = f.course_id {
                query.push(" AND mc.course_id = ?");
                query.push_bind(c);
            }
            if let Some(s) = f.sender_id {
                query.push(" AND m.sender_id = ?");
                query.push_bind(s);
            }
            if let Some(t) = f.title {
                query.push(" AND m.title LIKE ?");
                query.push_bind(format!("%{}%", t));
            }
        }
    let res = query
        .build_query_as()
        .fetch_all(pool)
        .await;
    res
    }
}
