use actix_web::{get, HttpResponse};

#[get("/notifications/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
