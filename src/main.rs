use actix_web::{http::header, middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use sqlx::mysql::MySqlPool;
use user::NewStudentData;
use user::NewTeacherData;
use user::NewUser;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_swagger_ui::Config;
use utoipa::Modify;


mod views;

use views::login::login;
use views::login::__path_login;

use views::register::create_user;
use views::register::__path_create_user;

use views::students::update_students;
use views::students::__path_update_students;

use views::teachers::update_teachers;
use views::teachers::__path_update_teachers;

use creation::create_grades;
use creation::__path_create_grades;

mod user;
mod jwt;
mod json;
mod creation;
mod functions;

use user::{User, Credentials, TeacherData, StudentData};
use jwt::Claims;
use creation::Grades;

#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        create_user,
        update_students,
        update_teachers,
        create_grades,
    ),
    components(
        schemas(User, Grades, Credentials, Claims, TeacherData, StudentData, NewUser, NewTeacherData, NewStudentData)
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
            utoipa::openapi::security::ApiKey::Cookie(utoipa::openapi::security::ApiKeyValue::new("jwt"))
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
    let pool = MySqlPool::connect("mysql://root:mili2009@localhost/goschool")
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
            .service(update_teachers)
            .service(update_students)
            /*.service(create_grades)*/
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
