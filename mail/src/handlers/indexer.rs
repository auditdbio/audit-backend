use actix_web::{get, HttpResponse};

#[get("/api/mail/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
