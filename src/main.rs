use actix_web::{http::header, middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use sqlx::mysql::MySqlPool;
use env_logger;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Config};
use utoipa::Modify;
use chrono::Datelike;

mod views;
mod user;
mod jwt;
mod json;

use views::login::{login, __path_login};
use views::register::{create_user, __path_create_user};
use views::create_homework::{create_homework, __path_create_homework};
use views::create_submission::{create_submission, __path_create_submission};
use user::{User, Credentials, NewUser};
use jwt::Claims;

#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        create_user,
        create_homework,
        create_submission,
    ),
    components(
        schemas(
            User,
            Credentials,
            Claims,
            NewUser,
        )
    ),
    tags(
        (name = "users", description = "User management endpoints"),
        (name = "auth", description = "Authentication endpoints")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let security_scheme = utoipa::openapi::security::SecurityScheme::ApiKey(
            utoipa::openapi::security::ApiKey::Cookie(
                utoipa::openapi::security::ApiKeyValue::new("jwt")
            )
        );
        openapi
            .components
            .as_mut()
            .unwrap()
            .add_security_scheme("cookieAuth", security_scheme);
    }
}

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
            .allowed_origin("http://localhost")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .supports_credentials();

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(json_conf.clone())
            .service(create_user)
            .service(login)
            .service(create_submission)
            .service(create_homework)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
                    .config(Config::default().with_credentials(true))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
