use actix_web::HttpResponse;
use sqlx::MySqlPool;
use crate::structs::*;
use crate::traits::{Delete, Get};

impl Delete for MySelf {
    async fn delete_assessment(&self, pool: &MySqlPool, assessment_id: u64) -> HttpResponse {
        // Solo admin o teacher dueño de la materia
        let is_authorized = match self.role {
            Role::admin => true,
            Role::teacher => {
                let exists: Result<bool, _> = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM assessments a JOIN subjects s ON a.subject_id = s.id WHERE a.id = ? AND s.teacher_id = ?)"
                )
                .bind(assessment_id)
                .bind(self.id)
                .fetch_one(pool)
                .await;
                exists.unwrap_or(false)
            },
            _ => false
        };
        
        if !is_authorized {
            return HttpResponse::Unauthorized().finish();
        }
        
        let res = sqlx::query("DELETE FROM assessments WHERE id = ?")
            .bind(assessment_id)
            .execute(pool)
            .await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn delete_grade(&self, pool: &MySqlPool, grade_id: u64) -> HttpResponse {
        // Solo admin o teacher dueño de la materia
        let is_authorized = match self.role {
            Role::admin => true,
            Role::teacher => {
                let exists: Result<bool, _> = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM grades g JOIN subjects s ON g.subject_id = s.id WHERE g.id = ? AND s.teacher_id = ?)"
                )
                .bind(grade_id)
                .bind(self.id)
                .fetch_one(pool)
                .await;
                exists.unwrap_or(false)
            },
            _ => false
        };
        
        if !is_authorized {
            return HttpResponse::Unauthorized().finish();
        }

        let res = sqlx::query("DELETE FROM grades WHERE id = ?")
            .bind(grade_id)
            .execute(pool)
            .await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn delete_message(&self, pool: &MySqlPool, message_id: u64) -> HttpResponse {
        // Solo admin, preceptor, o el sender
        let is_authorized = match self.role {
            Role::admin | Role::preceptor => true,
            _ => {
                let exists: Result<bool, _> = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM messages WHERE id = ? AND sender_id = ?)"
                )
                .bind(message_id)
                .bind(self.id)
                .fetch_one(pool)
                .await;
                exists.unwrap_or(false)
            }
        };
        
        if !is_authorized {
            return HttpResponse::Unauthorized().finish();
        }
        
        let res = sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(message_id)
            .execute(pool)
            .await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn delete_personal_data(&self, pool: &MySqlPool, user_id: u64) -> HttpResponse {
        // Solo admin puede borrar datos personales de otros
        if self.role != Role::admin {
            return HttpResponse::Unauthorized().finish();
        }
        let res = sqlx::query("DELETE FROM personal_data WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn delete_profile_picture(&self, pool: &MySqlPool, user_id: u64) -> HttpResponse {
        // Solo admin o el propio usuario
        if self.role != Role::admin && self.id != user_id {
            return HttpResponse::Unauthorized().finish();
        }
        
        let res = sqlx::query("UPDATE users SET photo = NULL WHERE id = ?")
            .bind(user_id)
            .execute(pool)
            .await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn delete_subject_message(&self, pool: &MySqlPool, subject_message_id: u64) -> HttpResponse {
        // Solo admin o teacher dueño de la materia
        let is_authorized = match self.role {
            Role::admin => true,
            Role::teacher => {
                let exists: Result<bool, _> = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM subject_messages sm JOIN subjects s ON sm.subject_id = s.id WHERE sm.id = ? AND s.teacher_id = ?)"
                )
                .bind(subject_message_id)
                .bind(self.id)
                .fetch_one(pool)
                .await;
                exists.unwrap_or(false)
            },
            _ => false
        };
        if !is_authorized {
            return HttpResponse::Unauthorized().finish();
        }
        //deleting file in case of existance
        let file_exists: Result<bool, _> = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM subject_messages WHERE id = ? AND file IS NOT NULL)"
        )
        .bind(subject_message_id)
        .fetch_one(pool)
        .await;
        if file_exists.unwrap_or(false) {
            let res: String = match sqlx::query_scalar("SELECT file FROM subject_messages WHERE id = ?")
                .bind(subject_message_id)
                .fetch_one(pool)
                .await {
                    Ok(file) => file,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
            };
            std::fs::remove_file(res).unwrap();
        }
                    

        let res = sqlx::query("DELETE FROM subject_messages WHERE id = ?")
            .bind(subject_message_id)
            .execute(pool)
            .await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn delete_submission(&self, pool: &MySqlPool, submission_id: u64) -> HttpResponse {
        // Solo admin o el propio estudiante
        let is_authorized = match self.role {
            Role::admin => true,
            Role::student => {
                let exists: Result<bool, _> = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM homework_submissions WHERE id = ? AND student_id = ?)"
                )
                .bind(submission_id)
                .bind(self.id)
                .fetch_one(pool)
                .await;
                exists.unwrap_or(false)
            },
            _ => false
        };
        if !is_authorized {
            return HttpResponse::Unauthorized().finish();
        }

        //deleting file in case of existance
        let file_exists: Result<bool, _> = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM homework_submissions WHERE id = ? AND file IS NOT NULL)"
        )
        .bind(submission_id)
        .fetch_one(pool)
        .await;
        if file_exists.unwrap_or(false) {
            let res: String = match sqlx::query_scalar("SELECT file FROM homework_submissions WHERE id = ?")
                .bind(submission_id)
                .fetch_one(pool)
                .await {
                    Ok(file) => file,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
            };
            std::fs::remove_file(res).unwrap();
        }

        let res = sqlx::query("DELETE FROM homework_submissions WHERE id = ?")
            .bind(submission_id)
            .execute(pool)
            .await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }
    async  fn delete_assistance(
            &self,
            pool: &MySqlPool,
            assistance_id: u64,
            ) -> HttpResponse {
         match self.role {
            Role::preceptor => {
                let courses = self.get_courses(pool).await.unwrap();
                let student_id: u64 = match sqlx::query_scalar("SELECT student_id FROM assisstance WHERE id = ?")
                    .bind(assistance_id)
                    .fetch_one(pool)
                .await{
                    Ok(s) => s,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
                };
                let student_course: u64 = match sqlx::query_scalar("SELECT course_id FROM users WHERE id = ?").bind(student_id).fetch_one(pool).await {
                    Ok(sc) => sc,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
                };
                let has_access = courses.iter().any(|course| course.id == student_course);

                if !has_access {
                    return HttpResponse::Unauthorized().finish();
                }

            },
            Role::admin => {},
            _ => {
                return HttpResponse::Unauthorized().finish();
            }
        }
        let result = sqlx::query("DELETE FROM assisstance WHERE id = ?")
            .bind(assistance_id)
            .execute(pool)
            .await;
        match result {
            Ok(_) => return HttpResponse::Ok().finish(),
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
        }
    }
    async fn delete_disciplinary_sanction(
            &self,
            pool: &MySqlPool,
            disciplinary_sanction_id: u64
        ) -> HttpResponse {
        match self.role {
            Role::preceptor => {
                let courses = self.get_courses(pool).await.unwrap();
                let student_id: u64 = match sqlx::query_scalar("SELECT student_id FROM disciplinary_sanctions WHERE id = ?")
                    .bind(disciplinary_sanction_id)
                    .fetch_one(pool)
                .await{
                    Ok(s) => s,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
                };
                let student_course: u64 = match sqlx::query_scalar("SELECT course_id FROM users WHERE id = ?").bind(student_id).fetch_one(pool).await {
                    Ok(sc) => sc,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
                };
                let has_access = courses.iter().any(|course| course.id == student_course);

                if !has_access {
                    return HttpResponse::Unauthorized().finish();
                }

            },
            Role::admin => {},
            _ => {
                return HttpResponse::Unauthorized().finish();
            }
        }
        let result = sqlx::query("DELETE FROM disciplinary_sanctions WHERE id = ?")
            .bind(disciplinary_sanction_id)
            .execute(pool)
            .await;
        match result {
            Ok(_) => return HttpResponse::Ok().finish(),
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
        }

    }
}
