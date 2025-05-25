use actix_web::{get, web, HttpRequest, HttpResponse, Responder, post};
use sqlx::mysql::MySqlPool;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::io::Write;
use tempfile::NamedTempFile;
use uuid::Uuid;
use std::fs;



use crate::jwt::validate;
use crate::structs::PhotoUrlResponse;
use crate::functions::cleanup_temp;

#[get("/api/v1/profile_pictures/")]
pub async fn get_profile_picture(req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder {
    
    let cookie = match req.cookie("jwt") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    
    let url = match token.claims.user.get_profile_picture(&pool).await {
        Ok(a) => a,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };
    
    HttpResponse::Ok().json(PhotoUrlResponse { url })
}

#[post("/api/v1/profile_pictures/")]
pub async fn post_profile_picture(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut task_submission: Multipart,
) -> impl Responder {
    let cookie = match req.cookie("jwt") {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let token = match validate(cookie.value()) {
        Ok(t) => t,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = token.claims.user.id;


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
            .bind(user_id as i32)
            .execute(pool.get_ref())
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
