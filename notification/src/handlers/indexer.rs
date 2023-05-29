use actix_web::{get, HttpResponse};

#[get("/api/notifications/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
