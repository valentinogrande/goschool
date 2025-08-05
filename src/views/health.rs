use actix_web::{get, HttpResponse, Responder};

#[get("/api/v1/health/")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
