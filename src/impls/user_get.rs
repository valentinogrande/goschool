use actix_web::web;
use chrono::Utc;
use sqlx::{MySql, MySqlPool, QueryBuilder};
use std::env;
use rand::seq::SliceRandom;

use crate::filters::*;
use crate::structs::*;
use crate::traits::Get;

impl Get for MySelf {
    async fn get_students(
        &self,
        pool: web::Data<MySqlPool>,
        filter: UserFilter,
    ) -> Result<Vec<PubUser>, sqlx::Error> {
        
    
        let mut query = sqlx::QueryBuilder::new(
            "SELECT users.id, users.photo, users.course_id, users.email, pd.full_name \
             FROM users \
             JOIN personal_data pd ON users.id = pd.user_id "
        );

        match &self.role {
            Role::teacher => {
                query.push("JOIN courses c ON users.course_id = c.id ");
                query.push("JOIN subjects s ON s.course_id = c.id ");
                query.push("WHERE s.teacher_id = ");
                query.push_bind(self.id);
            }
            Role::student => {
                query.push("WHERE users.id = ");
                query.push_bind(self.id);
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
                query.push("WHERE 1=1 ");
            }
        }

        if let Some(c) = filter.course {
            query.push(" AND users.course_id = ");
            query.push_bind(c);
        }

        if let Some(n) = filter.name {
            query.push(" AND pd.full_name LIKE ");
            query.push_bind(format!("%{}%", n));
        }

        if let Some(r) = filter.role {
            query.push(" AND users.id IN (SELECT user_id FROM roles WHERE role = ");
            query.push_bind(r);
            query.push(")");
        }

        let res: Result<Vec<PubUser>, sqlx::Error> = query
            .build_query_as::<PubUser>()
            .fetch_all(pool.as_ref())
            .await;

        res
    }

    async fn get_courses(&self, pool: &MySqlPool) -> Result<Vec<Course>, sqlx::Error> {
        
        let mut query = QueryBuilder::new(
            "SELECT c.* FROM courses c "
        );

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
        }

        // Agrupar por ID para evitar duplicados
        query.push(" GROUP BY c.id");

        let res = query.build_query_as::<Course>().fetch_all(pool).await;
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
        
        let mut query = QueryBuilder::new("SELECT s.*, c.name as course_name FROM subjects s JOIN courses c ON s.course_id = c.id ");
        
        match self.role {
            Role::teacher => {
                query.push("WHERE s.teacher_id =");
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
                query.push("WHERE c.preceptor_id =");
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
    
    async fn get_public_selfassessables(
        &self,
        pool: &MySqlPool,
        filter: SelfassessableFilter,
    ) -> Result<Vec<PublicSelfassessable>, sqlx::Error> {
        
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
                 WHERE DATE(a.due_date) = CURRENT_DATE() AND u.id =  "
        );

        query_builder.push_bind(self.id);
        if let Some(i) = filter.assessment_id {
            query_builder.push(" AND s.id = ");
            let self_id = self.get_selfassessable_id(pool, i).await?;
            query_builder.push_bind(self_id);
        }
        let query = query_builder.build_query_as::<Selfassessable>();
        let selfassessables = query.fetch_all(pool).await?;
            
        dbg!(&selfassessables);

        // Convertir a PublicSelfassessable con opciones randomizadas
        let mut public_selfassessables = Vec::new();
        let mut rng = rand::rng();
        
        for selfassessable in selfassessables {
            // Recopilar todas las opciones disponibles
            let mut options = vec![selfassessable.correct.clone(), selfassessable.incorrect1.clone()];
            
            if let Some(inc2) = &selfassessable.incorrect2 {
                options.push(inc2.clone());
            }
            if let Some(inc3) = &selfassessable.incorrect3 {
                options.push(inc3.clone());
            }
            if let Some(inc4) = &selfassessable.incorrect4 {
                options.push(inc4.clone());
            }
            
            // Randomizar el orden de las opciones
            options.shuffle(&mut rng);
            
            // Crear PublicSelfassessable con opciones randomizadas
            let public_selfassessable = PublicSelfassessable {
                id: selfassessable.id,
                question: selfassessable.question,
                op1: options[0].clone(),
                op2: options[1].clone(),
                op3: options.get(2).cloned(),
                op4: options.get(3).cloned(),
                op5: options.get(4).cloned(),
            };
            
            public_selfassessables.push(public_selfassessable);
        }
        
        Ok(public_selfassessables)
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
                 WHERE DATE(a.due_date) = CURRENT_DATE() AND u.id =  ",
        );

        query_builder.push_bind(self.id);

        if let Some(i) = filter.assessment_id {
            query_builder.push(" AND s.id = ");
            let self_id = self.get_selfassessable_id(pool, i).await?; 
            query_builder.push_bind(self_id);
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

        if let Some(i) = filter.assessment_id {
            query_builder.push(" AND selfassessable_id = ");
            let self_id = self.get_selfassessable_id(pool, i).await?;
            query_builder.push_bind(self_id);
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

        if let Some(i) = filter.assessment_id {
            query_builder.push(" AND selfassessable_id = ");
            let self_id = self.get_selfassessable_id(pool, i).await?;
            query_builder.push_bind(self_id);
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
    async fn get_timetables(
            &self,
            pool: &MySqlPool,
            filter: TimetableFilter)
        -> anyhow::Result<Vec<Timetable>, sqlx::Error> {
            let mut query = QueryBuilder::new("SELECT * FROM timetables t JOIN subjects s ON s.id = t.subject_id JOIN users u ON u.course_id = s.course_id JOIN courses c ON s.course_id = c.id");
            match self.role {
                Role::student => {
                    query.push("  WHERE u.id = ");
                    query.push_bind(self.id);
                }
                Role::admin => {
                    query.push(" WHERE 1=1");
                }
                Role::preceptor => {
                    query.push(" WHERE c.preceptor_id = ");
                    query.push_bind(self.id);
                }
                Role::teacher => {
                    query.push("  WHERE s.teacher_id = ");
                    query.push_bind(self.id);
                }
                Role::father => {
                    query.push(" JOIN families f ON f.student_id = u.id WHERE f.father_id = ");
                    query.push_bind(self.id);
                }
            };

            if let Some(i) = filter.day {
                query.push(" AND t.day = ");
                query.push_bind(i);
            }
            if let Some(s) = filter.subject_id {
                query.push(" AND s.id = ");
                query.push_bind(s);
            }
            if let Some(s) = filter.teacher_id {
                query.push(" AND s.teacher_id = ");
                query.push_bind(s);
            }
            if let Some(s) = filter.course_id {
                query.push(" AND c.id = ");
                query.push_bind(s);
            }
            

            let res = query.build_query_as().fetch_all(pool).await;
            res
    }
    async fn get_selfassessable_id(
        &self,
        pool: &MySqlPool,
        assessment_id: u64)
    -> Result<u64, sqlx::Error> {
        let res: u64 = match sqlx::query_scalar("SELECT id FROM selfassessables WHERE assessment_id = ?")
            .bind(assessment_id)
            .fetch_one(pool)
            .await {
            Ok(id) => id,
            Err(e) => return Err(e),
        };
        Ok(res)
    }
     async fn get_assistance(
            &self,
            pool: &MySqlPool,
            filter: AssistanceFilter
        ) -> Result<Vec<Assistance>, sqlx::Error> {
        let mut query = QueryBuilder::new("SELECT DISTINCT * FROM assistance a ");
        match self.role {
            Role::student => {
                query.push("WHERE a.student_id = ");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
            Role::teacher => {
                query.push("WHERE 1=2"); // blocking access to teachers
            }
            Role::father => {
                query.push("JOIN families f ON f.student_id = a.student_id WHERE f.father_id = ");
                query.push_bind(self.id);
            }
            Role::preceptor => {
                query.push("JOIN users u ON a.student_id = u.id JOIN courses c ON u.course_id = c.id WHERE c.preceptor_id = ");
                query.push_bind(self.id);
            }
        }
        if let Some(student_id) = filter.assistance_id {
                query.push(" AND a.student_id = ");
                query.push_bind(student_id);
        }
        if let Some(assistance_id) = filter.assistance_id {
            query.push(" AND a.id = ");
            query.push_bind(assistance_id);
        }
        if let Some(presence) = filter.presence {
            query.push(" AND a.presence = ");
            query.push_bind(presence);
        }
        if let Some(date) = filter.date {
            query.push(" AND a.date = ");
            query.push_bind(date);
        }
        let assistance = query.build_query_as().fetch_all(pool).await;
        assistance
    }
    async fn get_disciplinary_sanction(
            &self,
            pool: &MySqlPool,
            filter: DisciplinarySanctionFilter
        ) -> Result<Vec<DisciplinarySanction>, sqlx::Error> {

        let mut query = QueryBuilder::new("SELECT DISTINCT * FROM disciplinary_sanctions ds ");
        match self.role {
            Role::student => {
                query.push("WHERE ds.student_id = ");
                query.push_bind(self.id);
            }
            Role::admin => {
                query.push("WHERE 1=1");
            }
            Role::teacher => {
                query.push("WHERE 1=2"); // blocking access to teachers
            }
            Role::father => {
                query.push("JOIN families f ON f.student_id = ds.student_id WHERE f.father_id = ");
                query.push_bind(self.id);
            }
            Role::preceptor => {
                query.push("JOIN users u ON ds.student_id = u.id JOIN courses c ON u.course_id = c.id WHERE c.preceptor_id = ");
                query.push_bind(self.id);
            }
        }
        
        if let Some(ds_id) = filter.disciplinary_sanction_id {
            query.push(" AND ds.id = ");
            query.push_bind(ds_id);
        }
        if let Some(student_id) = filter.student_id {
            query.push(" AND ds.student_id = ");
            query.push_bind(student_id);
        }
        if let Some(sanction_type) = filter.sanction_type {
            query.push(" AND ds.sanction_type = ");
            query.push_bind(sanction_type);
        }
        
        let disciplinary_sanctions = query.build_query_as().fetch_all(pool).await;
        disciplinary_sanctions
    }
    async fn get_chats(
            &self,
            pool: &MySqlPool,
            filter: ChatFilter
        ) -> Result<Vec<Chat>, sqlx::Error> {
        
let mut query = QueryBuilder::new(r#"
    SELECT 
        c.id AS chat_id,
        c.name,
        c.photo,
        c.description,
        c.created_at,
        m.id AS last_message_id,
        m.message AS last_message,
        m.created_at AS last_message_time,
        CASE 
            WHEN EXISTS (
                SELECT 1 
                FROM chat_messages cm
                LEFT JOIN `reads` r 
                    ON r.message_id = cm.id AND r.reader_id = ?
                WHERE cm.chat_id = c.id AND r.id IS NULL
            ) THEN TRUE
            ELSE FALSE
        END AS has_unread
    FROM chats c
    JOIN chat_participants cp 
        ON c.id = cp.chat_id
    LEFT JOIN chat_messages m 
        ON m.id = (
            SELECT id 
            FROM chat_messages 
            WHERE chat_id = c.id 
            ORDER BY created_at DESC 
            LIMIT 1
        )
    WHERE cp.user_id = ?
"#);

        // bind the user_id twice: once for unread check, once for filter
        query.push_bind(self.id);
        query.push_bind(self.id);

        if let Some(n) = filter.name {
            query.push(" AND c.name LIKE ");
            query.push_bind(format!("%{}%", n));
        }

        query.push(" ORDER BY m.created_at IS NULL, m.created_at DESC");

        let chats = query.build_query_as().fetch_all(pool).await;

        chats

    }

    async fn get_chat_messages(
            &self,
            pool: &MySqlPool,
            filter: ChatMessageFilter
        ) -> Result<Vec<ChatMessage>, sqlx::Error> {
        
        let mut query = QueryBuilder::new(r#"
            SELECT * 
            FROM chat_message cm
            JOIN chats c ON cm.chat_id = c.id
            JOIN chat_participants cp ON c.id = cp.chat_id
            WHERE cp.user_id = ?
        "#);

        query.push_bind(self.id);

        if let Some(c) = filter.chat_id {
            query.push(" AND c.id = ?");
            query.push_bind(c);
        }

        if let Some(m) = filter.message {
            query.push(" AND cm.message LIKE ");
            query.push_bind(format!("%{}%", m));
        }

        let messages = query.build_query_as::<ChatMessage>().fetch_all(pool).await;
        let msg = messages?;

        for m in &msg {
            sqlx::query(
                "INSERT IGNORE INTO `reads` (message_id, reader_id) VALUES (?, ?)"
            )
            .bind(m.id)
            .bind(self.id)
            .execute(pool)
            .await?;
        }

        Ok(msg)
    }
}
