use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::mysql::MySqlPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use env_logger::Env;

mod views;

use views::login::login;
use views::login::__path_login;

use views::register::create_user;
use views::register::__path_create_user;

use views::students::update_students;
use views::students::__path_update_students;

use views::teachers::update_teachers;
use views::teachers::__path_update_teachers;


mod user;
mod jwt;
mod json;

use user::{User, Credentials, TeacherData, StudentData};
use jwt::Claims;

#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        create_user,
        update_students,
        update_teachers,
    ),
    components(
        schemas(User, Credentials, Claims, TeacherData, StudentData)
    ),
    tags(
        (name = "users", description = "User management endpoints"),
        (name = "auth", description = "Authentication endpoints")
    )
)]
struct ApiDoc;




#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = MySqlPool::connect("mysql://root:mili2009@localhost/goschool")
        .await
        .expect("Failed to connect to database");

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let json_conf = json::json_config();
    
    /*let pass = hash("admin", DEFAULT_COST).unwrap();
    let res = sqlx::query("INSERT INTO users (email, password, is_admin) VALUES ('admin',?,1)")
        .bind(pass)
        .execute(&pool)
        .await;*/

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(json_conf.clone())
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header())
            .service(create_user)
            .service(login)
            .service(update_teachers)
            .service(update_students)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
