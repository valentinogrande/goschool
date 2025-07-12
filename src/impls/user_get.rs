use actix_web::web;
use chrono::Utc;
use sqlx::{MySql, MySqlPool, QueryBuilder};
use std::env;

use crate::filters::{
    AssessmentFilter, GradeFilter, MessageFilter, SelfassessableFilter, SubjectFilter,
    SubjectMessageFilter, UserFilter,
};
use crate::structs::{
    Assessment, Course, Grade, Message, MySelf, PendingSelfassessableGrade, PersonalData,
    PublicPersonalData, Role, Selfassessable, SelfassessableResponse, Subject, SubjectMessage,
};
use crate::traits::Get;

impl Get for MySelf {
    async fn get_students(
        &self,
        pool: web::Data<MySqlPool>,
        filter: UserFilter,
    ) -> Result<Vec<u64>, sqlx::Error> {
        let mut query = sqlx::QueryBuilder::new("SELECT DISTINCT users.id FROM users ");

        match &self.role {
            Role::teacher => {
                query.push("JOIN courses c ON users.course_id = c.id ");
                query.push("JOIN subjects s ON s.course_id = c.id ");
                query.push("WHERE s.teacher_id = ");
                query.push_bind(self.id);
            }
            Role::student => {
                return Ok(vec![self.id]);
            }
            Role::preceptor => {
                query.push("JOIN courses c ON users.course_id = c.id ");
                query.push("WHERE c.preceptor_id = ");
                query.push_bind(self.id);
            }
            Role::father => {
                query.push("JOIN families f ON f.student_id = users.id ");
                query.push("WHERE f.father_id = ");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
        }
        if let Some(c) = filter.course {
            query.push(" AND users.course_id = ");
            query.push_bind(c);
        }

        if let Some(n) = filter.name {
            query.push(" AND EXISTS (SELECT 1 FROM personal_data pd WHERE pd.user_id = users.id AND pd.full_name LIKE )");
            query.push_bind(format!("%{}%", n));
        }
        let res = query
            .build_query_scalar::<u64>()
            .fetch_all(pool.as_ref())
            .await;

        res
    }

    async fn get_courses(&self, pool: &MySqlPool) -> Result<Vec<Course>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT * FROM courses c ");
        match self.role {
            Role::student => {
                query.push("JOIN users u ON c.id = u.course_id WHERE u.id = ");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
            Role::preceptor => {
                query.push("WHERE preceptor_id = ");
                query.push_bind(self.id);
            }
            Role::father => {
                query.push("JOIN users u ON c.id = u.course_id JOIN families f ON f.student_id = u.id WHERE f.father_id = ");
                query.push_bind(self.id);
            }
            Role::teacher => {
                query.push("JOIN subjects s ON c.id = s.course_id WHERE s.teacher_id = ");
                query.push_bind(self.id);
            }
        };

        let res = query.build_query_as().fetch_all(pool).await;
        res
    }
    async fn get_grades(
        &self,
        pool: &MySqlPool,
        filter: GradeFilter,
    ) -> Result<Vec<Grade>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT * FROM grades g ");
        match self.role {
            Role::student => {
                query.push("WHERE g.student_id =");
                query.push_bind(self.id);
            }
            Role::teacher => {
                query.push("JOIN subjects s ON g.subject_id = s.id WHERE s.teacher_id =");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
            Role::father => {
                query.push("JOIN families f ON g.student_id = f.student_id WHERE f.father_id =");
                query.push_bind(self.id);
            }
            Role::preceptor => {
                query.push("JOIN subjects s ON g.subject_id = s.id JOIN courses c ON s.course_id = c.id WHERE c.preceptor_id =");
                query.push_bind(self.id);
            }
        };
        if let Some(c) = filter.student_id {
            query.push(" AND student_id = ");
            query.push_bind(c);
        }
        if let Some(s) = filter.subject_id {
            query.push(" AND subject_id = ");
            query.push_bind(s);
        }
        if let Some(d) = filter.description {
            query.push(" AND description = ");
            query.push_bind(d);
        }
        let res = query.build_query_as().fetch_all(pool).await;
        res
    }
    async fn get_subjects(
        &self,
        pool: &MySqlPool,
        filter: SubjectFilter,
    ) -> Result<Vec<Subject>, sqlx::Error> {
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
                query.push("WHERE s.course_id = (SELECT course_id FROM users WHERE id = ");
                query.push_bind(self.id);
                query.push(")");
            }
            Role::preceptor => {
                query.push("JOIN courses c ON s.course_id = c.id WHERE c.preceptor_id =");
                query.push_bind(self.id);
            }
            Role::father => {
                query.push(
                    "JOIN users u ON s.course_id = u.course_id \
                 JOIN families f ON f.student_id = u.id \
                 WHERE f.father_id = ",
                );
                query.push_bind(self.id);
            }
        };
        if let Some(c) = filter.course_id {
            query.push(" AND s.course_id = ");
            query.push_bind(c);
        }
        if let Some(n) = filter.name {
            query.push(" AND s.name LIKE ");
            query.push_bind(n);
        }
        if let Some(t) = filter.teacher_id {
            query.push(" AND s.teacher_id = ");
            query.push_bind(t);
        }
        if let Some(i) = filter.subject_id {
            query.push(" AND s.id = ");
            query.push_bind(i);
        }
        let res = query.build_query_as::<Subject>().fetch_all(pool).await;
        res
    }

    async fn get_assessments(
        &self,
        pool: &MySqlPool,
        filter: AssessmentFilter,
        subject_filter: SubjectFilter,
        person_filter: UserFilter,
    ) -> Result<Vec<Assessment>, sqlx::Error> {
        let mut query = QueryBuilder::new(
            "SELECT a.* FROM assessments a \
             JOIN subjects s ON a.subject_id = s.id",
        );

        // Rol y permisos: condiciones base
        let mut where_started = false;

        let mut add_where = |q: &mut QueryBuilder<'_, MySql>, condition: &str| {
            if !where_started {
                q.push(" WHERE ");
                where_started = true;
            } else {
                q.push(" AND ");
            }
            q.push(condition);
        };

        match self.role {
            Role::teacher => {
                add_where(&mut query, "s.teacher_id = ");
                query.push_bind(self.id);
            }
            Role::admin => {
                // no filtro necesario
            }
            Role::father => {
                let subject_ids: Vec<u64> = sqlx::query_scalar(
                    "SELECT s.id FROM subjects s
                     JOIN users u ON s.course_id = u.course_id
                     JOIN families f ON f.student_id = u.id
                     WHERE f.father_id = ?",
                )
                .bind(self.id)
                .fetch_all(pool)
                .await?;

                if subject_ids.is_empty() {
                    return Ok(vec![]);
                }

                add_where(&mut query, "a.subject_id IN (");
                let mut separated = query.separated(", ");
                for id in subject_ids {
                    separated.push_bind(id);
                }
                query.push(")");
            }
            Role::student => {
                let subject_ids: Vec<u64> = sqlx::query_scalar(
                    "SELECT s.id FROM subjects s
                     JOIN users u ON s.course_id = u.course_id
                     WHERE u.id = ?",
                )
                .bind(self.id)
                .fetch_all(pool)
                .await?;

                if subject_ids.is_empty() {
                    return Ok(vec![]);
                }

                add_where(&mut query, "a.subject_id IN (");
                let mut separated = query.separated(", ");
                for id in subject_ids {
                    separated.push_bind(id);
                }
                query.push(")");
            }
            Role::preceptor => {
                let subject_ids: Vec<u64> = sqlx::query_scalar(
                    "SELECT s.id FROM subjects s
                     JOIN courses c ON s.course_id = c.id
                     WHERE c.preceptor_id = ?",
                )
                .bind(self.id)
                .fetch_all(pool)
                .await?;

                if subject_ids.is_empty() {
                    return Ok(vec![]);
                }

                add_where(&mut query, "a.subject_id IN (");
                let mut separated = query.separated(", ");
                for id in subject_ids {
                    separated.push_bind(id);
                }
                query.push(")");
            }
        }

        // Filtros de materia (subject)
        if let Some(course_id) = subject_filter.course_id {
            add_where(&mut query, "s.course_id = ");
            query.push_bind(course_id);
        }
        if let Some(ref name) = subject_filter.name {
            add_where(&mut query, "s.name LIKE ");
            query.push_bind(format!("%{}%", name));
        }
        if let Some(teacher_id) = subject_filter.teacher_id {
            add_where(&mut query, "s.teacher_id = ");
            query.push_bind(teacher_id);
        }
        if let Some(id) = subject_filter.subject_id {
            add_where(&mut query, "s.id = ");
            query.push_bind(id);
        }

        // Filtros de persona (usuario asociado a la evaluación)
        if let Some(ref full_name) = person_filter.name {
            add_where(
                &mut query,
                "EXISTS (
                    SELECT 1 FROM personal_data pd
                    WHERE pd.user_id = a.user_id AND pd.full_name LIKE ",
            );
            query.push_bind(format!("%{}%", full_name));
            query.push(")");
        }

        if let Some(course_id) = person_filter.course {
            add_where(
                &mut query,
                "EXISTS (
                    SELECT 1 FROM users u
                    WHERE u.id = a.user_id AND u.course_id = ",
            );
            query.push_bind(course_id);
            query.push(")");
        }

        if let Some(user_id) = person_filter.user_id {
            add_where(&mut query, "a.user_id = ");
            query.push_bind(user_id);
        }

        // Filtros propios de la evaluación
        if let Some(true) = filter.due {
            let now = Utc::now().naive_utc();
            add_where(&mut query, "a.due_date >= ");
            query.push_bind(now);
        }

        if let Some(ref task) = filter.task {
            add_where(&mut query, "a.task LIKE ");
            query.push_bind(format!("%{}%", task));
        }

        let result = query.build_query_as::<Assessment>().fetch_all(pool).await;

        result
    }

    async fn get_personal_data(&self, pool: &MySqlPool) -> Result<PersonalData, sqlx::Error> {
        let res = sqlx::query_as("SELECT * FROM personal_data WHERE user_id = ?")
            .bind(self.id)
            .fetch_one(pool)
            .await;

        res
    }

    async fn get_public_personal_data(
        &self,
        pool: &MySqlPool,
        filter: UserFilter,
    ) -> Result<Vec<PublicPersonalData>, sqlx::Error> {
        let mut query = QueryBuilder::new(
            "SELECT pd.full_name,u.photo FROM personal_data pd JOIN users u ON pd.user_id = u.id",
        );

        if let Some(n) = filter.name {
            query.push(" WHERE pd.full_name LIKE ");
            query.push_bind(format!("%{}%", n));
        }
        if let Some(i) = filter.user_id {
            query.push(" AND pd.user_id = ");
            query.push_bind(i);
        }
        if let Some(c) = filter.course {
            query.push(" AND u.course_id = ");
            query.push_bind(c);
        }

        let res = query.build_query_as().fetch_all(pool).await;

        res
    }

    async fn get_profile_picture(&self, pool: &MySqlPool) -> Result<String, sqlx::Error> {
        let photo_filename: String =
            match sqlx::query_scalar("SELECT photo FROM users WHERE id = ?")
                .bind(self.id)
                .fetch_optional(pool)
                .await
            {
                Ok(Some(path)) => path,
                Ok(None) => {
                    let base_url = env::var("BASE_URL").expect("BASE_URL must be set");
                    format!("{}uploads/profile_pictures/default.jpg", base_url)
                }
                Err(e) => return Err(e),
            };

        let url = format!("{}", photo_filename);
        Ok(url)
    }

    async fn get_messages(
        &self,
        pool: &MySqlPool,
        filter: MessageFilter,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let mut query = QueryBuilder::new(
            "SELECT DISTINCT m.* FROM messages m
             JOIN message_courses mc ON mc.message_id = m.id ",
        );
        match self.role {
            Role::student => {
                query.push("JOIN users u ON u.course_id = mc.course_id WHERE u.id = ");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
            Role::father => {
                query.push("JOIN users u ON u.course_id = mc.course_id JOIN families f ON f.student_id = u.id WHERE f.father_id = ");
                query.push_bind(self.id);
            }
            Role::teacher => {
                query.push("JOIN subjects s ON mc.course_id = s.course_id WHERE s.teacher_id = ");
                query.push_bind(self.id);
            }
            Role::preceptor => {
                query.push("JOIN courses c ON mc.course_id = c.id WHERE c.preceptor_id = ");
                query.push_bind(self.id);
            }
        };

        if let Some(c) = filter.course_id {
            query.push(" AND mc.course_id = ");
            query.push_bind(c);
        }
        if let Some(s) = filter.sender_id {
            query.push(" AND m.sender_id = ");
            query.push_bind(s);
        }
        if let Some(t) = filter.title {
            query.push(" AND m.title LIKE ");
            query.push_bind(format!("%{}%", t));
        }

        let res = query.build_query_as().fetch_all(pool).await;
        res
    }
    async fn get_selfassessables(
        &self,
        pool: &MySqlPool,
        filter: SelfassessableFilter,
    ) -> anyhow::Result<Vec<crate::structs::Selfassessable>, sqlx::Error> {
        if self.role != Role::student {
            return Err(sqlx::Error::Protocol(
                "Only students can get selfassessables".into(),
            ));
        }

        let mut query_builder = QueryBuilder::new(
            "SELECT st.* FROM selfassessable_tasks st
                 JOIN selfassessables s ON s.id = st.selfassessable_id 
                 JOIN assessments a ON a.id = s.assessment_id
                 JOIN subjects sj ON sj.id = a.subject_id
                 JOIN users u ON u.course_id = sj.course_id
                 WHERE u.id =  ",
        );

        query_builder.push_bind(self.id);

        if let Some(i) = filter.selfassessable_id {
            query_builder.push(" AND s.id = ");
            query_builder.push_bind(i);
        }

        let query = query_builder.build_query_as::<Selfassessable>();

        let selfassessables = query.fetch_all(pool).await?;

        Ok(selfassessables)
    }
    async fn get_selfassessables_responses(
        &self,
        pool: &MySqlPool,
        filter: SelfassessableFilter,
    ) -> anyhow::Result<Vec<SelfassessableResponse>, sqlx::Error> {
        if self.role != Role::student {
            return Err(sqlx::Error::Protocol(
                "Only students can get selfassessables".into(),
            ));
        }

        let mut query_builder =
            QueryBuilder::new("SELECT * FROM selfassessable_submissions WHERE student_id = ");

        query_builder.push_bind(self.id);

        if let Some(i) = filter.selfassessable_id {
            query_builder.push(" AND selfassessable_id = ");
            query_builder.push_bind(i);
        }

        let query = query_builder.build_query_as::<SelfassessableResponse>();

        let responses = query.fetch_all(pool).await?;

        Ok(responses)
    }
    async fn get_pending_selfassessables_grades(
        &self,
        pool: &MySqlPool,
        filter: SelfassessableFilter,
    ) -> anyhow::Result<Vec<PendingSelfassessableGrade>, sqlx::Error> {
        let mut query_builder = QueryBuilder::new("SELECT * FROM selfassessable_pending_grades pg");
        match self.role {
            Role::student => {
                return Err(sqlx::Error::Protocol(
                    "Only teachers can get selfassessables grades".into(),
                ));
            }
            Role::preceptor => {
                return Err(sqlx::Error::Protocol(
                    "Only teachers can get selfassessables grades".into(),
                ));
            }
            Role::father => {
                return Err(sqlx::Error::Protocol(
                    "Only teachers can get selfassessables grades".into(),
                ));
            }
            Role::teacher => {
                query_builder.push(" JOIN selfassessables s ON s.id = pg.selfassessable_id JOIN assessments a ON a.id = s.assessment_id JOIN subjects sj ON sj.id = a.subject_id JOIN users u ON u.course_id = sj.course_id WHERE u.id = ");
                query_builder.push_bind(self.id);
            }
            Role::admin => {
                query_builder.push(" WHERE 1=1");
            }
        }

        if let Some(i) = filter.selfassessable_id {
            query_builder.push(" AND selfassessable_id = ");
            query_builder.push_bind(i);
        }

        let query = query_builder.build_query_as::<PendingSelfassessableGrade>();

        let selfassessables = query.fetch_all(pool).await?;

        Ok(selfassessables)
    }
    async fn get_subject_messages(
        &self,
        pool: &MySqlPool,
        filter: SubjectMessageFilter,
    ) -> anyhow::Result<Vec<SubjectMessage>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT * FROM subject_messages sm");
        match self.role {
            Role::student => {
                query.push(" JOIN subjects s ON s.id = sm.subject_id JOIN users u ON u.course_id = s.course_id WHERE u.id = ");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push(" WHERE 1=1");
            }
            Role::preceptor => {
                query.push(" JOIN subjects s ON s.id = sm.subject_id JOIN courses c ON c.id = s.course_id WHERE c.preceptor_id = ");
                query.push_bind(self.id);
            }
            Role::teacher => {
                query.push(" JOIN subjects s ON s.id = sm.subject_id WHERE s.teacher_id = ");
                query.push_bind(self.id);
            }
            Role::father => {
                query.push(" JOIN subjects s ON s.id = sm.subject_id JOIN users u ON u.course_id = s.course_id JOIN families f ON f.student_id = u.id WHERE f.father_id = ");
                query.push_bind(self.id);
            }
        };

        if let Some(i) = filter.subject_message_id {
            query.push(" AND sm.id = ");
            query.push_bind(i);
        }
        if let Some(s) = filter.subject_id {
            query.push(" AND sm.subject_id = ");
            query.push_bind(s);
        }
        if let Some(s) = filter.sender_id {
            query.push(" AND sm.sender_id = ");
            query.push_bind(s);
        }

        let res = query.build_query_as().fetch_all(pool).await;
        res
    }
}
