use actix_web::{
    get,
    web::{self, Json},
};
use common::context::Context;

use crate::service::ping::{self, Service, Status};

#[get("/api/status")]
pub async fn status(context: Context, services: web::Data<Vec<Service>>) -> Json<Status> {
    let status = ping::status(context, &services).await;
    Json(status)
}
