use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use sqlx::mysql::MySqlPool;
use sqlx::Row;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod user;
mod jwt;

use user::{RespondPersonalData, PersonalData, NewUser, Credentials};
use jwt::{Claims, get_validator};

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

#[utoipa::path(
    post,
    path = "/api/v1/register",
    request_body(content = NewUser, description = "User registration data", content_type = "application/json"),
    responses(
        (status = 201, description = "User created successfully"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/api/v1/register/")]
async fn create_user(pool: web::Data<MySqlPool>, user: web::Json<NewUser>) -> impl Responder {
    let hashed_pass = hash(&user.password, DEFAULT_COST);
    if let Err(e) = hashed_pass {
        return HttpResponse::InternalServerError().json(e.to_string())
    }
    else {
        let hashed_pass = hashed_pass.unwrap();
        let result = sqlx::query("INSERT INTO user (FullName, password, email) VALUES (?,?,?)")
            .bind(user.username.clone())
            .bind(hashed_pass)
            .bind(user.email.clone())
            .execute(pool.get_ref())
            .await;

        match result {
            Ok(_) => HttpResponse::Created().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string())
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/login",
    request_body(content = Credentials, description = "User credentials", content_type = "application/json"),
    responses(
        (status = 200, description = "Login successful", body = String),
        (status = 401, description = "Invalid credentials")
    )
)]
#[post("/api/v1/login/")]
async fn login(pool: web::Data<MySqlPool>, creds: web::Json<Credentials>) -> impl Responder {
    let password_from_db = sqlx::query("SELECT userid,password FROM user WHERE email = ?")
        .bind(creds.email.clone())
        .fetch_one(pool.get_ref())
        .await;

    if let Ok(record) = password_from_db {
        let password = record.get::<String, &str>("password");
        if verify(&creds.password, &password).unwrap_or(false) {
            let claims = Claims::new(record.get::<i32, &str>("userid") as usize);
            let secret = "prod_secret";
            let token = encode(
                &Header::new(Algorithm::HS256),
                &claims,
                &EncodingKey::from_secret(secret.as_ref()),
            );

            HttpResponse::Ok().json(token.unwrap())
        } else {
            HttpResponse::Unauthorized().json("Invalid credentials")
        }
    } else {
        HttpResponse::Unauthorized().json("Invalid credentials")
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/update_personal_data",
    request_body(content = PersonalData, description = "Personal data to update", content_type = "application/json"),
    responses(
        (status = 201, description = "Personal data updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/api/v1/update_personal_data/")]
async fn update_personal_data(
    pool: web::Data<MySqlPool>,
    data: web::Json<PersonalData>) -> impl Responder {
        
    let validation = get_validator();
    let secret = "prod_secret";

    let decode = decode::<Claims>(
        data.jwt.as_str(),
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    );

    match decode {
        Ok(token_data) => {
            let userid = token_data.claims.subject as i32;
            match sqlx::query(
                "INSERT INTO person (nombre_completo, edad, mensaje, user_id) VALUES (?,?,?,?)")
                .bind(data.nombre_completo.clone())
                .bind(data.edad)
                .bind(data.mensaje.clone())
                .bind(userid)
                .execute(pool.get_ref())
                .await {
                    Ok(_) => HttpResponse::Created().json("Personal data was updated"),
                    Err(e) => HttpResponse::InternalServerError().json(e.to_string())
                }
        },
        Err(e) => HttpResponse::Unauthorized().json(e.to_string())
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/get_personal_data",
    request_body(content = String, description = "JWT token", content_type = "application/json"),
    responses(
        (status = 200, description = "Success", body = RespondPersonalData),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
#[get("/api/v1/get_personal_data/")]
async fn get_personal_data(pool: web::Data<MySqlPool>, jwt: web::Json<String>) -> impl Responder {
    let validation = get_validator();
    let secret = "prod_secret";

    match decode::<Claims>(
        jwt.as_str(),
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    ) {
        Ok(token_data) => {
            let userid = token_data.claims.subject as i32;
            match sqlx::query(
                "SELECT nombre_completo, edad, mensaje FROM person WHERE user_id = ?")
                .bind(userid)
                .fetch_one(pool.get_ref())
                .await {
                    Ok(record) => {
                        let name = record.get::<String, &str>("nombre_completo");
                        let age = record.get::<i32, &str>("edad");
                        let msg = record.get::<String, &str>("mensaje");
                        let personal_data = RespondPersonalData::new(name, age, msg);
                        HttpResponse::Ok().json(personal_data)
                    },
                    Err(_) => HttpResponse::NotFound().json("Personal data not found")
                }
        },
        Err(e) => HttpResponse::Unauthorized().json(e.to_string())
    }
}

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
