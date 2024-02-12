use actix_web::{get, HttpResponse};

#[get("/mail/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
