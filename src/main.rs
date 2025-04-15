use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use sqlx::mysql::MySqlPool;
use env_logger;
use chrono::Datelike;
use actix_files::Files;

mod views;
mod user;
mod jwt;
mod json;
mod sqlx_fn;

use views::login::login;
use views::register::register;
use views::create_assessment::create_assessment;
use views::assign_grade::assign_grade;
use views::create_submission::create_submission;
use views::upload_profile_picture::upload_profile_picture;
use views::get_profile_picture::get_profile_picture;
use views::register_testing_users::register_users;
use views::verify_token::verify_token;
use views::get_assessmets::{get_assessments, get_assessments_by_id};
use views::get_grades::{get_grades, get_grades_by_id};
use views::get_role::get_role;
use views::get_roles::get_roles;
use views::logout::logout;

use jwt::Claims;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    dotenv::dotenv().ok();
    let mut db_url = std::env::var("DATABASE_URL").expect("database url should be setted");
    let actual_year = chrono::Utc::now().year();
    db_url.push_str(actual_year.to_string().as_str());

    let pool = MySqlPool::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let json_conf = json::json_config();
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() 
            .allow_any_method() 
            .allow_any_header()
            .supports_credentials();
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(json_conf.clone())
            .service(Files::new("/uploads/profile_pictures", "./uploads/profile_pictures").index_file("404"))
            .service(register)
            .service(verify_token)
            .service(login)
            .service(logout)
            .service(create_submission)
            .service(get_assessments)
            .service(get_assessments_by_id)
            .service(get_grades)
            .service(get_grades_by_id)
            .service(create_assessment)
            .service(upload_profile_picture)
            .service(get_profile_picture)
            .service(get_role)
            .service(get_roles)
            .service(assign_grade)
            .service(register_users) // for creating testing users.
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
