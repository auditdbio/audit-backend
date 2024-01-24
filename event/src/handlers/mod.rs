use actix_web::{get, HttpResponse};

pub mod event;

#[get("/notifications/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
