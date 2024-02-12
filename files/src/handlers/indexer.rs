use actix_web::{get, HttpResponse};

#[get("/files/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
