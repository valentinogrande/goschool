use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use sqlx::mysql::MySqlPool;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::{fs::File, io::Write};
use uuid::Uuid;

use crate::jwt::validate;

#[derive(serde::Deserialize, serde::Serialize,utoipa::ToSchema)]
struct Task{
    id: i32,
    grade: i32,
}


#[utoipa::path(
    post,
    path = "/api/v1/create_submission/",
    request_body(content = Task, description = "task submission data", content_type = "multipart//form-data"),
    responses(
        (status = 200, description = "submission created successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "bad request"),
        (status = 500, description = "InternalServerError"),
    )
)]

#[post("/api/v1/create_submission/")]
pub async fn create_submission(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut task_submission: Multipart,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value().to_string()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.subject;

    let user_grade = match sqlx::query_as::<_, (i32,)>(
        "SELECT grade_id FROM students WHERE user_id = ?"
    )
    .bind(user_id as i32)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok((g,)) => g,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let mut saved_file_name: Option<String> = None;
    let mut task_id: Option<i32> = None;
    let mut path: Option<String> = None;
    let mut content_file: Vec<u8> = Vec::new();

    while let Some(item) = task_submission.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };

        match field.name() {
            Some("task") => {
                let filename = field
                    .content_disposition()
                    .and_then(|cd| cd.get_filename().map(sanitize_filename::sanitize));

                let filename = match filename {
                    Some(name) => {
                        if !(name.ends_with(".pdf") || name.ends_with(".docx")) {
                            return HttpResponse::BadRequest().body("Invalid file type");
                        }
                        name
                    }
                    None => return HttpResponse::BadRequest().body("Missing filename"),
                };

                let extension = filename.split('.').last().unwrap_or("docx");
                let unique_name = format!("{}.{}", Uuid::new_v4(), extension);
                path = Some(format!("./uploads/submissions/{}", unique_name));
                let mut total_size = 0;

                while let Some(chunk) = field.next().await {
                    let chunk = chunk.unwrap();
                    total_size += chunk.len();
                    if total_size > 10 * 1024 * 1024 {
                        return HttpResponse::BadRequest().body("file too large");
                    }
                    content_file.extend_from_slice(&chunk);
                }

                saved_file_name = Some(unique_name);
            }
            Some("task_id") => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk.unwrap());
                }

                let text = String::from_utf8(data).unwrap_or_default();
                match text.trim().parse::<i32>() {
                    Ok(id) => task_id = Some(id),
                    Err(_) => return HttpResponse::BadRequest().body("Invalid task ID"),
                }
            }
            _ => {}
        }
    }

    let task_id = match task_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("Missing task_id"),
    };

    let task_grade = match sqlx::query_scalar::<_, i32>("SELECT grade FROM tasks where id = ?")
        .bind(task_id)
        .fetch_one(pool.get_ref())
        .await{
        Ok(g) => g,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    

    if task_grade != user_grade {
        return HttpResponse::Unauthorized().finish();
    }

    let already_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM task_submissions WHERE student = ? AND task = ?)"
    )
    .bind(user_id as i32)
    .bind(task_id)
    .fetch_one(pool.get_ref())
    .await;

    match already_exists {
        Ok(true) => return HttpResponse::BadRequest().body("You already submitted this task"),
        Ok(false) => {}
        Err(_) => return HttpResponse::InternalServerError().finish(),
    }

    let mut file = match File::create(path.as_ref().unwrap()) {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if let Err(_) = file.write_all(&content_file) {
        return HttpResponse::InternalServerError().finish();
    }
    if let Err(_) = file.flush() {
        return HttpResponse::InternalServerError().finish();
    }

    match saved_file_name {
        Some(path) => {
            let result = sqlx::query(
                "INSERT INTO task_submissions (path, student, task) VALUES (?, ?, ?)"
            )
            .bind(path)
            .bind(user_id as i32)
            .bind(task_id)
            .execute(pool.get_ref())
            .await;

            if result.is_err() {
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok().body("Submission created")
        }
        _ => HttpResponse::BadRequest().body("Missing data"),
    }
}
