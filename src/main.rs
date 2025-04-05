
use actix_web::{http::header, middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use sqlx::mysql::MySqlPool;
use env_logger;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Config};
use utoipa::Modify;

mod views;
mod user;
mod jwt;
mod json;
mod creation;
mod functions;

use views::create_submission::{create_submission, __path_create_submission};
use views::create_task::{create_task, __path_create_task, NewTask};
use views::login::{login, __path_login};
use views::register::{create_user, __path_create_user};
use views::students::{update_students, __path_update_students};
use views::teachers::{update_teachers, __path_update_teachers};
use creation::{__path_create_grades, Grades};
use user::{User, Credentials, TeacherData, StudentData, NewUser, NewTeacherData, NewStudentData};
use jwt::Claims;

#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        create_user,
        update_students,
        update_teachers,
        create_grades,
        create_task,
        create_submission,
    ),
    components(
        schemas(
            User,
            Grades,
            Credentials,
            Claims,
            TeacherData,
            StudentData,
            NewUser,
            NewTeacherData,
            NewStudentData,
            NewTask,
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
    // Crear pool de conexión a MySQL
    let pool = MySqlPool::connect("mysql://root:mili2009@localhost/goschool")
        .await
        .expect("Failed to connect to database");

    // Inicializar logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Config JSON (deserialización, payload size, etc.)
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
            .service(update_teachers)
            .service(update_students)
            .service(create_submission)
            .service(create_task)
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
