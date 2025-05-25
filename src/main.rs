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
mod routes;
mod filters;
mod structs;
mod functions;

use jwt::Claims;

use routes::register_services;

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
            .configure(register_services)
   })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
