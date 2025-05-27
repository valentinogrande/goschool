use sqlx::MySqlPool;
use anyhow::Result;
use futures::future::join_all;

use crate::structs::{MySelf, Role, AssessmentType, Payload};
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
}
