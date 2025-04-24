use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::io::Write;
use tempfile::NamedTempFile;
use uuid::Uuid;
use std::fs;

use crate::user::Role;
use crate::jwt::validate;


fn cleanup_temp(path: &Option<String>) {
    if let Some(p) = path {
        let _ = std::fs::remove_file(p);
    }
}

#[post("/api/v1/create_submission/")]
pub async fn create_submission(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut homework_submission: Multipart,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.subject as u64;

    let user_course = match sqlx::query_as::<_, (u64,)>(
        "SELECT course_id FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok((g,)) => g,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let mut saved_file_name: Option<String> = None;
    let mut homework_id: Option<u64> = None;
    let mut final_path: Option<String> = None;
    let mut temp_path: Option<String> = None;

    while let Some(item) = homework_submission.next().await {
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

 
    let role = token.claims.role;
    if role != Role::student {
        cleanup_temp(&temp_path);
        return HttpResponse::Unauthorized().finish();
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
        .fetch_one(pool.get_ref())
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
    .fetch_one(pool.get_ref())
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
    .bind(user_id as u64)
    .bind(homework_id)
    .fetch_one(pool.get_ref())
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
            .bind(user_id as u64)
            .bind(homework_id)
            .execute(pool.get_ref())
            .await;

            if result.is_err() {
                cleanup_temp(&temp_path);
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok().body("Submission created")
        }
        None => {
            cleanup_temp(&temp_path);
            HttpResponse::BadRequest().body("Missing data")
        }
    }
}
