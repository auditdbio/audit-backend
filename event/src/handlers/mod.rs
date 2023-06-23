use actix_web::{get, HttpResponse};

pub mod event;

#[get("/api/notifications/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
