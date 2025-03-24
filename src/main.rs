use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::mysql::MySqlPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod views;

use views::login::login;
use views::login::__path_login;

use views::register::create_user;
use views::register::__path_create_user;

use views::update_personal_data::update_personal_data;
use views::update_personal_data::__path_update_personal_data;

use views::get_personal_data::get_personal_data;
use views::get_personal_data::__path_get_personal_data;

mod user;
mod jwt;

use user::{RespondPersonalData, PersonalData, NewUser, Credentials};
use jwt::{Claims, validate};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_personal_data,
        update_personal_data,
        login,
        create_user
    ),
    components(
        schemas(NewUser, Credentials, PersonalData, RespondPersonalData, Claims)
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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header())
            .service(create_user)
            .service(login)
            .service(update_personal_data)
            .service(get_personal_data)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
