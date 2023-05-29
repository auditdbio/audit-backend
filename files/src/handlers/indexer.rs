use actix_web::{get, HttpResponse};

#[get("/api/files/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
