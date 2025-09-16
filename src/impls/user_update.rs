use actix_multipart::Multipart;
use actix_web::HttpResponse;
use sqlx::MySqlPool;
use crate::structs::*;
use crate::traits::{Update, Get};

impl Update for MySelf {
    async fn update_assessment(&self, pool: &MySqlPool, assessment_id: u64, data: UpdateAssessment) -> HttpResponse {
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
        let mut query = String::from("UPDATE assessments SET ");
        let mut first = true;
        if let Some(subject_id) = data.subject_id {
            if !first { query.push_str(", "); } query.push_str("subject_id = "); query.push_str(&subject_id.to_string()); first = false;
        }
        if let Some(ref task) = data.task {
            if !first { query.push_str(", "); } query.push_str("task = '"); query.push_str(task); query.push('\''); first = false;
        }
        if let Some(due_date) = data.due_date {
            if !first { query.push_str(", "); } query.push_str("due_date = '"); query.push_str(&due_date.to_string()); query.push('\''); first = false;
        }
        if let Some(ref type_) = data.type_ {
            if !first { query.push_str(", "); } query.push_str("type = '"); query.push_str(&format!("{:?}", type_)); query.push('\''); first = false;
        }
        if first { return HttpResponse::BadRequest().body("No fields to update"); }
        query.push_str(" WHERE id = ?");
        let res = sqlx::query(&query).bind(assessment_id).execute(pool).await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn update_grade(&self, pool: &MySqlPool, grade_id: u64, data: UpdateGrade) -> HttpResponse {
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
        let mut query = String::from("UPDATE grades SET ");
        let mut first = true;
        if let Some(ref description) = data.description {
            if !first { query.push_str(", "); } query.push_str("description = '"); query.push_str(description); query.push('\''); first = false;
        }
        if let Some(ref grade) = data.grade {
            if !first { query.push_str(", "); } query.push_str("grade = "); query.push_str(&grade.to_string()); first = false;
        }
        if let Some(student_id) = data.student_id {
            if !first { query.push_str(", "); } query.push_str("student_id = "); query.push_str(&student_id.to_string()); first = false;
        }
        if let Some(subject_id) = data.subject_id {
            if !first { query.push_str(", "); } query.push_str("subject_id = "); query.push_str(&subject_id.to_string()); first = false;
        }
        if let Some(assessment_id) = data.assessment_id {
            if !first { query.push_str(", "); } query.push_str("assessment_id = "); query.push_str(&assessment_id.to_string()); first = false;
        }
        if let Some(ref grade_type) = data.grade_type {
            if !first { query.push_str(", "); } query.push_str("grade_type = '"); query.push_str(&format!("{:?}", grade_type)); query.push('\''); first = false;
        }
        if first { return HttpResponse::BadRequest().body("No fields to update"); }
        query.push_str(" WHERE id = ?");
        let res = sqlx::query(&query).bind(grade_id).execute(pool).await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn update_message(&self, pool: &MySqlPool, message_id: u64, data: UpdateMessage) -> HttpResponse {
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
        let mut query = String::from("UPDATE messages SET ");
        let mut first = true;
        if let Some(ref title) = data.title {
            if !first { query.push_str(", "); } query.push_str("title = '"); query.push_str(title); query.push('\''); first = false;
        }
        if let Some(ref message) = data.message {
            if !first { query.push_str(", "); } query.push_str("message = '"); query.push_str(message); query.push('\''); first = false;
        }
        if first { return HttpResponse::BadRequest().body("No fields to update"); }
        query.push_str(" WHERE id = ?");
        let res = sqlx::query(&query).bind(message_id).execute(pool).await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn update_personal_data(&self, pool: &MySqlPool, user_id: u64, data: UpdatePersonalData) -> HttpResponse {
        // Solo admin puede actualizar datos personales de otros
        if self.role != Role::admin {
            return HttpResponse::Unauthorized().finish();
        }
        let mut query = String::from("UPDATE personal_data SET ");
        let mut first = true;
        if let Some(ref full_name) = data.full_name {
            if !first { query.push_str(", "); } query.push_str("full_name = '"); query.push_str(full_name); query.push('\''); first = false;
        }
        if let Some(ref phone_number) = data.phone_number {
            if !first { query.push_str(", "); } query.push_str("phone_number = '"); query.push_str(phone_number); query.push('\''); first = false;
        }
        if let Some(ref address) = data.address {
            if !first { query.push_str(", "); } query.push_str("address = '"); query.push_str(address); query.push('\''); first = false;
        }
        if let Some(birth_date) = data.birth_date {
            if !first { query.push_str(", "); } query.push_str("birth_date = '"); query.push_str(&birth_date.to_string()); query.push('\''); first = false;
        }
        if first { return HttpResponse::BadRequest().body("No fields to update"); }
        query.push_str(" WHERE user_id = ?");
        let res = sqlx::query(&query).bind(user_id).execute(pool).await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn update_profile_picture(&self, pool: &MySqlPool, user_id: u64, multipart: Multipart) -> HttpResponse {
        // Solo admin o el propio usuario
        if self.role != Role::admin && self.id != user_id {
            return HttpResponse::Unauthorized().finish();
        }
        use crate::parse_multipart::parse_multipart;
        use std::str;
        let hashmap = match parse_multipart(
            multipart,
            Some(&["jpg", "jpeg", "png"]),
            Some(&["image/jpeg", "image/png"]),
            "uploads/profile_pictures",
        ).await {
            Ok(h) => h,
            Err(e) => return HttpResponse::BadRequest().json(format!("Invalid upload: {}", e)),
        };
        let file_name = hashmap
            .get("file")
            .and_then(|bytes| str::from_utf8(bytes).ok());
        let result = sqlx::query("UPDATE users SET photo = ? WHERE id = ?")
            .bind(file_name)
            .bind(user_id)
            .execute(pool)
            .await;
        if result.is_err() {
            return HttpResponse::InternalServerError().finish();
        }
        HttpResponse::Ok().finish()
    }

    async fn update_subject_message(&self, pool: &MySqlPool, subject_message_id: u64, data: UpdateSubjectMessage) -> HttpResponse {
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
        let mut query = String::from("UPDATE subject_messages SET ");
        let mut first = true;
        if let Some(ref title) = data.title {
            if !first { query.push_str(", "); } query.push_str("title = '"); query.push_str(title); query.push('\''); first = false;
        }
        if let Some(ref content) = data.content {
            if !first { query.push_str(", "); } query.push_str("content = '"); query.push_str(content); query.push('\''); first = false;
        }
        if let Some(ref type_) = data.type_ {
            if !first { query.push_str(", "); } query.push_str("type = '"); query.push_str(&format!("{:?}", type_)); query.push('\''); first = false;
        }
        if first { return HttpResponse::BadRequest().body("No fields to update"); }
        query.push_str(" WHERE id = ?");
        let res = sqlx::query(&query).bind(subject_message_id).execute(pool).await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }

    async fn update_submission(&self, pool: &MySqlPool, submission_id: u64, data: UpdateSubmission) -> HttpResponse {
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
        let mut query = String::from("UPDATE homework_submissions SET ");
        let mut first = true;
        if let Some(ref path) = data.path {
            if !first { query.push_str(", "); } query.push_str("path = '"); query.push_str(path); query.push('\''); first = false;
        }
        if let Some(student_id) = data.student_id {
            if !first { query.push_str(", "); } query.push_str("student_id = "); query.push_str(&student_id.to_string()); first = false;
        }
        if let Some(task_id) = data.task_id {
            if !first { query.push_str(", "); } query.push_str("task_id = "); query.push_str(&task_id.to_string()); first = false;
        }
        if first { return HttpResponse::BadRequest().body("No fields to update"); }
        query.push_str(" WHERE id = ?");
        let res = sqlx::query(&query).bind(submission_id).execute(pool).await;
        match res {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    }
    async fn update_assistance(
            &self,
            pool: &MySqlPool,
            assistance_id: u64,
            data: UpdateAssistance
        ) -> HttpResponse {
        match self.role {
            Role::preceptor => {
                let courses = self.get_courses(pool).await.unwrap();
                let student_id = data.student_id;
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
        let result = sqlx::query("UPDATE assistance SET presence = ?, date = ? WHERE id = ?")
            .bind(data.presence)
            .bind(data.date)
            .bind(assistance_id)
            .execute(pool)
            .await;
            match result {
                Ok(_) => return HttpResponse::Ok().finish(),
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
            }
    }
    async fn update_disciplinary_sanction(
            &self,
            pool: &MySqlPool,
            disciplinary_sanction_id: u64,
            data: UpdateDisciplinarySanction
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
        let result = sqlx::query("UPDATE disciplinary_sanctions SET sanction_type = ?, date = ?, quantity = ?, description = ? WHERE id = ?")
            .bind(data.sanction_type)
            .bind(data.date)
            .bind(data.quantity)
            .bind(data.description)
            .bind(disciplinary_sanction_id)
            .execute(pool)
            .await;
            match result {
                Ok(_) => return HttpResponse::Ok().finish(),
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
            }
    }
}
