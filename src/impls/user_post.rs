use actix_web::HttpResponse;
use sqlx::MySqlPool;
use futures::future::join_all;
use futures_util::StreamExt;
use std::io::Write;
use tempfile::NamedTempFile;
use uuid::Uuid;
use std::fs;

use crate::functions::cleanup_temp;
use crate::structs::{MySelf, Role, AssessmentType, Payload, NewGrade, NewMessage};
use crate::filters::SubjectFilter;
use crate::traits::{Get, Post};

impl Post for MySelf {
     async fn post_assessment(
        &self,
        pool: &MySqlPool,
        payload: Payload,
    ) -> HttpResponse {

        match self.role {
            Role::teacher => {
                
                let mut filter = SubjectFilter::new();
                filter.id = Some(payload.newtask.subject);
                
                let subjects = match self.get_subjects(pool, Some(filter)).await{
                    Ok(s) => s,
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
                if subjects.is_empty() {
                    return HttpResponse::Unauthorized().finish();
                }
            }
            Role::admin => {}
            _ => {return HttpResponse::Unauthorized().finish()}
        };


        if payload.newtask.type_ == AssessmentType::Selfassessable{
        
            let selfassessable = match &payload.newselfassessable {
                Some(a) => a,
                None => return HttpResponse::BadRequest().json("Missing selfassessable"),
            };

            if !(selfassessable.validate()){
                return HttpResponse::BadRequest().json("Invalid selfassessable");
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
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        };
            let assessment_id = insert_result.last_insert_id();
        
            let assessable = match sqlx::query("INSERT INTO selfassessables (assessment_id) VALUES (?)").bind(assessment_id).execute(pool).await {
                Ok(r)=>r,
                Err(e)=>return HttpResponse::InternalServerError().json(e.to_string()),
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
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                }
            }

            return HttpResponse::Created().finish();
        } else{

            let insert_result = sqlx::query("INSERT INTO assessments (task, subject_id, type, due_date) VALUES (?, ?, ?, ?)")
            .bind(&payload.newtask.task)
            .bind(payload.newtask.subject)
            .bind(&payload.newtask.type_)
            .bind(&payload.newtask.due_date)
            .execute(pool)
            .await;

            match insert_result {
               Ok(_) => HttpResponse::Created().finish(),
               Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        
        }
    }
    async fn post_grade(
            &self,
            pool: &MySqlPool,
            grade: NewGrade,
        ) -> HttpResponse {
        match self.role {
            Role::admin => {}
            Role::teacher => {
                let teacher_subject: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM subjects WHERE teacher_id = ? AND id = ?)")
                .bind(self.id)
                .bind(grade.subject)
                .fetch_one(pool)
                .await {
                Ok(s) => s,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };
            if !teacher_subject {
                return HttpResponse::Unauthorized().finish();
            }
        }
            _ => {return HttpResponse::Unauthorized().finish()}
        };
        
        let course = match sqlx::query_scalar::<_, u64>("SELECT course_id FROM subjects WHERE id = ?")
            .bind(grade.subject)
            .fetch_one(pool)
            .await{
            Ok(c) => c,
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        };

        let student_course: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND course_id = ?)")
            .bind(grade.student_id)
            .bind(course)
            .fetch_one(pool)
            .await {
            Ok(s) => s,
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        };
        if !student_course{
            return HttpResponse::Unauthorized().finish();
        }
    
        if let Some(assessment_id) = grade.assessment_id{
        
            let assessment_verify: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assessments WHERE id = ? AND subject_id = ?)")
                .bind(assessment_id)
                .bind(grade.subject)
                .fetch_one(pool)
                .await{
                Ok(s) => s,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };
            if !assessment_verify{
                return HttpResponse::Unauthorized().finish();
            }
            let assessment_already_exixts: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM grades WHERE assessment_id = ? AND student_id = ? )")
            .bind(assessment_id)
            .bind(grade.student_id)
            .fetch_one(pool)
            .await {
                Ok(s) => s,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };
            if assessment_already_exixts{
                return HttpResponse::Unauthorized().finish();
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
                return HttpResponse::InternalServerError().finish();
            }
            else {
                return HttpResponse::Created().finish();
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
            return HttpResponse::InternalServerError().finish();
        }
        else {
            return HttpResponse::Created().finish();
        }
    }
    async fn post_message(
            &self,
            pool: &MySqlPool,
            message: NewMessage,
        ) -> HttpResponse {
        // cheking if courses are valid
        let courses: Vec<u64> = message
            .courses
            .split(',')
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .collect();
        for course in courses.iter() {
            if *course <= 0 || *course > 36 {
                return HttpResponse::BadRequest().json("Invalid courses");
            } 
        }

        match self.role {
            Role::admin => {}
            Role::preceptor => {
                let preceptor_courses: Vec<u64> = match self.get_courses(pool).await {
                    Ok(c) => c.iter().map(|c| c.id).collect(),
                    Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
                };
                if !preceptor_courses.iter().all(|&course| courses.contains(&course)) {
                    return HttpResponse::Unauthorized().finish();
                }
            }
            _ => {}
            
        };

        let message_id = match sqlx::query("INSERT INTO messages (message, sender_id, title) VALUES (?, ?, ?)").bind(&message.message).bind(self.id).bind(&message.title).execute(pool).await {
            Ok(ref result) => result.last_insert_id(),
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        };

        for course in courses.iter() {
            let _insert_result = match sqlx::query("INSERT INTO message_courses (course_id, message_id) VALUES (?,?)")
            .bind(course)
            .bind(message_id)
            .execute(pool)
            .await {
                Ok(r) =>  r,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            };
        }
        HttpResponse::Created().finish()
    }

    async fn post_profile_picture(
            &self,
            pool: &MySqlPool,
            mut task_submission: actix_multipart::Multipart
        ) -> HttpResponse {

        let mut saved_file_name: Option<String> = None;
        let mut final_path: Option<String> = None;
        let mut temp_path: Option<String> = None;

        while let Some(item) = task_submission.next().await {
            let mut field = match item {
                Ok(f) => f,
                Err(_) => return HttpResponse::BadRequest().finish(),
            };

            match field.name() {
                Some("image") => {
                    let filename = field
                        .content_disposition()
                        .and_then(|cd| cd.get_filename().map(sanitize_filename::sanitize));

                    let filename = match filename {
                        Some(mut name) => {
                            let supported_extensions = [".png",".jpg",".jpeg",".webp"];
                            name = name.to_lowercase();
                            if !(supported_extensions.iter().any(|&ext| name.ends_with(ext))) {
                                return HttpResponse::BadRequest().body("Invalid file type");
                            }
                            name
                        }
                        None => return HttpResponse::BadRequest().body("Missing filename"),
                    };

                    let extension = filename.split('.').last().unwrap();
                    let unique_name = format!("{}.{}", Uuid::new_v4(), extension);
                    saved_file_name = Some(unique_name.clone());
                    let upload_path = format!("./uploads/profile_pictures/{}", unique_name);
                    final_path = Some(upload_path);

                    let mut temp_file = match NamedTempFile::new() {
                        Ok(f) => f,
                        Err(_) => return HttpResponse::InternalServerError().finish(),
                    };

                    let mut total_size = 0;
                    while let Some(chunk) = field.next().await {
                        let chunk = match chunk {
                            Ok(c) => c,
                            Err(_) => {
                                cleanup_temp(&temp_path);
                                return HttpResponse::InternalServerError().finish();
                            },
                        };

                        total_size += chunk.len();
                        if total_size > 10 * 1024 * 1024 {
                            cleanup_temp(&temp_path);
                            return HttpResponse::BadRequest().body("File too large");
                        }

                        if let Err(_) = temp_file.write_all(&chunk) {
                            cleanup_temp(&temp_path);
                            return HttpResponse::InternalServerError().finish();
                        }
                    }

                    let temp = match temp_file.into_temp_path().keep() {
                        Ok(pathbuf) => pathbuf,
                        Err(_) => {
                            cleanup_temp(&temp_path);
                            return HttpResponse::InternalServerError().finish();
                        },
                    };

                    temp_path = Some(temp.to_string_lossy().to_string());
                }

                _ => {}
            }
        }


        if let (Some(temp), Some(final_path)) = (&temp_path, &final_path) {
            if let Err(_) = fs::rename(temp, final_path) {
                cleanup_temp(&temp_path);
                return HttpResponse::InternalServerError().body("Failed to store image");
            }
        } else {
            cleanup_temp(&temp_path);
            return HttpResponse::BadRequest().body("Missing image data");
        }

        match saved_file_name {
            Some(path) => {
                let result = sqlx::query(
                    "UPDATE users SET photo = ? WHERE id = ?"
                )
                .bind(path)
                .bind(self.id)
                .execute(pool)
                .await;

                if result.is_err() {
                    cleanup_temp(&temp_path);
                    return HttpResponse::InternalServerError().finish();
                }

                HttpResponse::Ok().body("image uploaded succesfully")
            }
            None => {
                cleanup_temp(&temp_path);
                HttpResponse::BadRequest().body("Missing data")
            }
        }
    }
    async fn post_submission(
            &self,
            pool: &MySqlPool,
            mut task_submission: actix_multipart::Multipart
        ) -> HttpResponse {
        
        match self.role {
            Role::student => {}
            _ => return HttpResponse::Unauthorized().finish(),
        };
    
        let user_course = self.get_courses(pool).await;
        let user_course = match user_course {
            Ok(c) if !c.is_empty() => c[0].id,
            Ok(_) => return HttpResponse::NotFound().json("No course found"),
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        };

        let mut saved_file_name: Option<String> = None;
        let mut homework_id: Option<u64> = None;
        let mut final_path: Option<String> = None;
        let mut temp_path: Option<String> = None;

        while let Some(item) = task_submission.next().await {
            let mut field = match item {
                Ok(f) => f,
                Err(_) => return HttpResponse::BadRequest().finish(),
            };

            match field.name() {
                Some("homework") => {
                    let filename = field
                        .content_disposition()
                        .and_then(|cd| cd.get_filename().map(sanitize_filename::sanitize));

                    let filename = match filename {
                        Some(mut name) => {
                            let supported_extensions = [".pdf",".docx"];
                            name = name.to_lowercase();
                            if !(supported_extensions.iter().any(|&ext| name.ends_with(ext))) {
                                return HttpResponse::BadRequest().body("Invalid file type");
                            }
                            name
                        }
                        None => return HttpResponse::BadRequest().body("Missing filename"),
                    };

                    let extension = filename.split('.').last().unwrap_or("docx");
                    let unique_name = format!("{}.{}", Uuid::new_v4(), extension);
                    saved_file_name = Some(unique_name.clone());
                    let upload_path = format!("./uploads/submissions/{}", unique_name);
                    final_path = Some(upload_path);

                    let mut temp_file = match NamedTempFile::new() {
                        Ok(f) => f,
                        Err(_) => return HttpResponse::InternalServerError().finish(),
                    };

                    let mut total_size = 0;
                    while let Some(chunk) = field.next().await {
                        let chunk = match chunk {
                            Ok(c) => c,
                            Err(_) => {
                                cleanup_temp(&temp_path);
                                return HttpResponse::InternalServerError().finish();
                            },
                        };

                        total_size += chunk.len();
                        if total_size > 10 * 1024 * 1024 {
                            cleanup_temp(&temp_path);
                            return HttpResponse::BadRequest().body("File too large");
                        }

                        if let Err(_) = temp_file.write_all(&chunk) {
                            cleanup_temp(&temp_path);
                            return HttpResponse::InternalServerError().finish();
                        }
                    }

                    let temp = match temp_file.into_temp_path().keep() {
                        Ok(pathbuf) => pathbuf,
                        Err(_) => {
                            cleanup_temp(&temp_path);
                            return HttpResponse::InternalServerError().finish();
                        },
                    };

                    temp_path = Some(temp.to_string_lossy().to_string());
                }

                Some("homework_id") => {
                    let mut data = Vec::new();
                    while let Some(chunk) = field.next().await {
                        data.extend_from_slice(&chunk.unwrap());
                    }

                    let text = String::from_utf8(data).unwrap_or_default();
                    match text.trim().parse::<u64>() {
                        Ok(id) => homework_id = Some(id),
                        Err(_) => {
                            cleanup_temp(&temp_path);
                            return HttpResponse::BadRequest().body("Invalid homework ID");
                        }
                    }
                }

                _ => {}
            }
        }

        let homework_id = match homework_id {
            Some(id) => id,
            None => {
                cleanup_temp(&temp_path);
                return HttpResponse::BadRequest().body("Missing homework_id");
            }
        };

        let res: (String, u64) = match sqlx::query_as("SELECT type, subject_id FROM assessments WHERE id = ?")
            .bind(homework_id)
            .fetch_one(pool)
            .await{
            Ok(g) => g,
            Err(_) => {
                cleanup_temp(&temp_path);
                return HttpResponse::InternalServerError().finish();
            }
        };

        if res.0 != "homework" {
            cleanup_temp(&temp_path);
            return HttpResponse::BadRequest().body("submission are only valid for homeworks");
        }
        let task_course = match sqlx::query_scalar::<_, u64>(
            "SELECT course_id FROM subjects WHERE id = ?"
        )
        .bind(res.1)
        .fetch_one(pool)
        .await{
            Ok(g) => g,
            Err(_) => {
                cleanup_temp(&temp_path);
                return HttpResponse::InternalServerError().finish();
            }
        };

        if task_course != user_course {
            cleanup_temp(&temp_path);
            return HttpResponse::Unauthorized().finish();
        }

        let already_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM homework_submissions WHERE student_id = ? AND task_id = ?)"
        )
        .bind(self.id)
        .bind(homework_id)
        .fetch_one(pool)
        .await;

        match already_exists {
            Ok(true) => {
                cleanup_temp(&temp_path);
                return HttpResponse::BadRequest().body("You already submitted this homework");
            }
            Ok(false) => {}
            Err(_) => {
                cleanup_temp(&temp_path);
                return HttpResponse::InternalServerError().finish();
            }
        }

        if let (Some(temp), Some(final_path)) = (&temp_path, &final_path) {
            if let Err(_) = fs::rename(temp, final_path) {
                cleanup_temp(&temp_path);
                return HttpResponse::InternalServerError().body("Failed to store file");
            }
        } else {
            cleanup_temp(&temp_path);
            return HttpResponse::BadRequest().body("Missing file data");
        }

        match saved_file_name {
            Some(path) => {
                let result = sqlx::query(
                    "INSERT INTO homework_submissions (path, student_id, task_id) VALUES (?, ?, ?)"
                )
                .bind(path)
                .bind(self.id)
                .bind(homework_id)
                .execute(pool)
                .await;

                if result.is_err() {
                    cleanup_temp(&temp_path);
                    return HttpResponse::InternalServerError().finish();
                }

                return HttpResponse::Ok().body("Submission created");
            }
            None => {
                cleanup_temp(&temp_path);
                return HttpResponse::BadRequest().body("Missing data");
            }
        }
    }
}
