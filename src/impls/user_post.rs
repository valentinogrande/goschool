use sqlx::MySqlPool;
use anyhow::Result;
use futures::future::join_all;

use crate::structs::{MySelf, Role, AssessmentType, Payload, NewGrade, NewMessage};
use crate::filters::SubjectFilter;
use crate::traits::{Get, Post};

impl Post for MySelf {
     async fn post_assessment(
        &self,
        pool: &MySqlPool,
        payload: Payload,
    ) -> Result<String> {

        match self.role {
            Role::teacher => {
                
                let mut filter = SubjectFilter::new();
                filter.id = Some(payload.newtask.subject);
                
                let subjects = match self.get_subjects(pool, Some(filter)).await{
                    Ok(s) => s,
                    Err(e) => return Err(e.into()),
                };
                if subjects.is_empty() {
                    return Err(anyhow::Error::msg("Unauthorized"));
                }
            }
            Role::admin => {}
            _ => {return Err(anyhow::Error::msg("Unauthorized"))}
        };


        if payload.newtask.type_ == AssessmentType::Selfassessable{
        
            let selfassessable = match &payload.newselfassessable {
                Some(a) => a,
                None => return Err(anyhow::Error::msg("Missing selfassessable")),
            };

            if !(selfassessable.validate()){
                return Err(anyhow::Error::msg("Invalid selfassessable"));
            }

            let insert_result = match sqlx::query("INSERT INTO assessments (task, subject_id, type, due_date) VALUES (?, ?, ?, ?)")
            .bind(&payload.newtask.task)
            .bind(payload.newtask.subject)
            .bind(&payload.newtask.type_)
            .bind(&payload.newtask.due_date)
            .execute(pool)
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(anyhow::Error::msg("Database error")),
        };
            let assessment_id = insert_result.last_insert_id();
        
            let assessable = match sqlx::query("INSERT INTO selfassessables (assessment_id) VALUES (?)").bind(assessment_id).execute(pool).await {
                Ok(r)=>r,
                Err(_)=>return Err(anyhow::Error::msg("Database error")),
            };
            let assessable_id = assessable.last_insert_id();
            let mut queries = selfassessable.generate_query(assessable_id);

            let results = join_all(
                queries.iter_mut().map(|q| {
                    q.build().execute(pool)  
                })
            ).await;
            for res in results {
                match res {
                    Ok(_) => {},
                    Err(e) => return Err(anyhow::Error::msg(e.to_string())),
                }
            }

            return Ok("selfassessable created".to_string())
        } else{

            let insert_result = sqlx::query("INSERT INTO assessments (task, subject_id, type, due_date) VALUES (?, ?, ?, ?)")
            .bind(&payload.newtask.task)
            .bind(payload.newtask.subject)
            .bind(&payload.newtask.type_)
            .bind(&payload.newtask.due_date)
            .execute(pool)
            .await;

            match insert_result {
               Ok(_) => Ok("assessment created".to_string()),
               Err(_) => Err(anyhow::Error::msg("Database error")),
            }
        
        }
    }
    async fn post_grade(
            &self,
            pool: &MySqlPool,
            grade: NewGrade,
        ) -> Result<String> {
        match self.role {
            Role::admin => {}
            Role::teacher => {
                let teacher_subject: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM subjects WHERE teacher_id = ? AND id = ?)")
                .bind(self.id)
                .bind(grade.subject)
                .fetch_one(pool)
                .await {
                Ok(s) => s,
                Err(_) => return Err(anyhow::Error::msg("Database error")),
            };
            if !teacher_subject {
                return Err(anyhow::Error::msg("Unauthorized"));
            }
        }
            _ => {return Err(anyhow::Error::msg("Unauthorized"))}
        };
        
        let course = match sqlx::query_scalar::<_, u64>("SELECT course_id FROM subjects WHERE id = ?")
            .bind(grade.subject)
            .fetch_one(pool)
            .await{
            Ok(c) => c,
            Err(_) => return Err(anyhow::Error::msg("Database error")),
        };

        let student_course: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND course_id = ?)")
            .bind(grade.student_id)
            .bind(course)
            .fetch_one(pool)
            .await {
            Ok(s) => s,
            Err(_) => return Err(anyhow::Error::msg("Database error")),
        };
        if !student_course{
            return Err(anyhow::Error::msg("Unauthorized"));
        }
    
        if let Some(assessment_id) = grade.assessment_id{
        
            let assessment_verify: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assessments WHERE id = ? AND subject_id = ?)")
                .bind(assessment_id)
                .bind(grade.subject)
                .fetch_one(pool)
                .await{
                Ok(s) => s,
                Err(_) => return Err(anyhow::Error::msg("Database error")),
            };
            if !assessment_verify{
                return Err(anyhow::Error::msg("Unauthorized"));
            }
            let assessment_already_exixts: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM grades WHERE assessment_id = ? AND student_id = ? )")
            .bind(assessment_id)
            .bind(grade.student_id)
            .fetch_one(pool)
            .await {
                Ok(s) => s,
                Err(_) => return Err(anyhow::Error::msg("Database error")),
            };
            if assessment_already_exixts{
                return Err(anyhow::Error::msg("Already exists"));
            }
            let result = sqlx::query("INSERT INTO grades (assessment_id, student_id, grade_type, description, grade, subject_id) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(assessment_id)
                .bind(grade.student_id)
                .bind(&grade.grade_type)
                .bind(&grade.description)
                .bind(grade.grade)
                .bind(grade.subject)
                .execute(pool)
                .await;
            if result.is_err() {
                return Err(anyhow::Error::msg("Database error"));
            }
            else {
                return Ok("grade created".to_string());
            }
        }
         let result = sqlx::query("INSERT INTO grades (student_id, grade_type, description, grade, subject_id) VALUES (?, ?, ?, ?, ?)")
            .bind(grade.student_id)
            .bind(&grade.grade_type)
            .bind(&grade.description)
            .bind(grade.grade)
            .bind(grade.subject)
            .execute(pool)
            .await;
        if result.is_err() {
            return Err(anyhow::Error::msg("Database error"));
        }
        else {
            return Ok("grade created".to_string());
        }
    }
    async fn post_message(
            &self,
            pool: &MySqlPool,
            message: NewMessage,
        ) -> Result<String> {
        // cheking if courses are valid
        let courses: Vec<u64> = message
            .courses
            .split(',')
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .collect();
        for course in courses.iter() {
            if *course <= 0 || *course > 36 {
                return Err(anyhow::Error::msg("Invalid course"));
            } 
        }

        match self.role {
            Role::admin => {}
            Role::preceptor => {
                let preceptor_courses: Vec<u64> = match self.get_courses(pool).await {
                    Ok(c) => c.iter().map(|c| c.id).collect(),
                    Err(e) => return Err(e.into()),
                };
                if !preceptor_courses.iter().all(|&course| courses.contains(&course)) {
                    return Err(anyhow::Error::msg("Unauthorized"));
                }
            }
            _ => {}
            
        };



        let message_id = match sqlx::query("INSERT INTO messages (message, sender_id, title) VALUES (?, ?, ?)").bind(&message.message).bind(self.id).bind(&message.title).execute(pool).await {
            Ok(ref result) => result.last_insert_id(),
            Err(_) => return Err(anyhow::Error::msg("Database error")),
        };

        for course in courses.iter() {
            let _insert_result = match sqlx::query("INSERT INTO message_courses (course_id, message_id) VALUES (?,?)")
            .bind(course)
            .bind(message_id)
            .execute(pool)
            .await {
                Ok(r) =>  r,
                Err(_) => return Err(anyhow::Error::msg("Database error")),
            };
        }
        Ok("message created".to_string())
    }
}
