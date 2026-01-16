use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpServer, middleware::Logger, web};
use chrono::Datelike;
use env_logger;
use sqlx::mysql::MySqlPool;

mod cron;
mod filters;
mod parse_multipart;
mod impls;
mod json;
mod jwt;
mod routes;
mod structs;
mod traits;
mod views;
mod email;
mod websocket;

use jwt::Claims;

use cron::start_cron_task;
use routes::register_services;
use websocket::ChatConnectionManager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let mut db_url = std::env::var("DATABASE_URL").expect("database url should be setted");
    let actual_year = chrono::Utc::now().year();
    db_url.push_str(actual_year.to_string().as_str());

    let pool = MySqlPool::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let pool_for_cron = pool.clone();
    tokio::spawn(async move {
        start_cron_task(pool_for_cron).await;
    });

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let json_conf = json::json_config();

    // Initialize WebSocket connection manager for chat
    let chat_manager = web::Data::new(ChatConnectionManager::new());
    log::info!("Chat WebSocket manager initialized");

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
            .app_data(chat_manager.clone())
            .app_data(json_conf.clone())
            .service(Files::new("/uploads/profile_pictures", "./uploads/profile_pictures").index_file("404"))
            .service(Files::new("/uploads/files", "./uploads/files").index_file("404"))
            .service(Files::new("/uploads/submissions", "./uploads/submissions").index_file("404"))
            .service(Files::new("/uploads/chat_files", "./uploads/chat_files").index_file("404"))
            .configure(register_services)
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}
