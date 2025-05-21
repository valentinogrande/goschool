use serde::{Serialize, Deserialize};
use sqlx::{MySqlPool, QueryBuilder};
use actix_web::web;

use crate::filters::{GradeFilter, UserFilter};
use crate::structs::{Grade, Role};

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
        filter: UserFilter,
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

        if let Some(c) = filter.course {
            query.push(" AND users.course_id = ?");
            query.push_bind(c);
        }

        if let Some(n) = filter.name {
            query.push(" AND EXISTS (SELECT 1 FROM personal_data pd WHERE pd.user_id = users.id AND pd.full_name LIKE ?)");
            query.push_bind(format!("%{}%", n));
        }

        let res = query
            .build_query_scalar::<u64>()
            .fetch_all(pool.as_ref())
            .await;

        res
    }
    
    pub async fn get_courses(&self, pool: &MySqlPool) -> Result<Vec<u64>, sqlx::Error> {
        match self.role {
            Role::student => {
                sqlx::query_scalar::<sqlx::MySql, u64>(
                    "SELECT course_id FROM users WHERE id = ?",
                )
                .bind(self.id)
                .fetch_one(pool)
                .await
                .map(|r| vec![r])
            },
            Role::admin => {
                sqlx::query_scalar::<sqlx::MySql, u64>(
                    "SELECT id FROM courses",
                )
                .fetch_all(pool)
                .await
            },
            Role::preceptor => {
                sqlx::query_scalar::<sqlx::MySql, u64>(
                    "SELECT id FROM courses WHERE preceptor_id = ?",
                )
                .bind(self.id)
                .fetch_all(pool)
                .await
            },
            Role::father => {
                sqlx::query_scalar::<sqlx::MySql, u64>(
                    "SELECT u.course_id FROM users u JOIN families f ON f.student_id = u.id WHERE f.father_id = ?",
                )
                .bind(self.id)
                .fetch_all(pool)
                .await
            },
            Role::teacher => {
                sqlx::query_scalar::<sqlx::MySql, u64>(
                    "SELECT c.id FROM courses c JOIN subjects s ON s.course_id = c.id WHERE s.teacher_id = ?",
                )
                .bind(self.id)
                .fetch_all(pool)
                .await
            },
        }
    }
    pub async fn get_grades(&self, pool: web::Data<MySqlPool>, filter: 
    GradeFilter) -> Result<Vec<Grade>, sqlx::Error>{
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
        if let Some(c) = filter.student_id {
            query.push(" AND student_id = ?");
            query.push_bind(c);
        }
        if let Some(s) = filter.subject_id {
            query.push(" AND subject_id = ?");
            query.push_bind(s);
        }
        if let Some(d) = filter.description {
            query.push(" AND description = ?");
            query.push_bind(d);
        }
        let res = query
            .build_query_as()
            .fetch_all(pool.as_ref())
            .await;
        res
    }

}



#[derive(Serialize,Deserialize)]
pub struct User{
    pub id: u64,
    pub password: String,
    pub email: String,
    pub role: Role,
    pub last_login: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewUser{
    pub password: String,
    pub email: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials{
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct CredentialsRole{
    pub email: String,
    pub password: String,
    pub role: Role,
}
