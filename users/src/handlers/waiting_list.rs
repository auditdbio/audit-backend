use actix_web::{get, HttpResponse, web::Path};
use common::context::Context;

use crate::service::waiting_list::WaitingListService;

#[get("/user/run_action/{secret}")]
pub async fn run_action(context: Context, secret: Path<String>) -> HttpResponse {
    WaitingListService::new(context).run_action(secret.into_inner()).await.unwrap();
    HttpResponse::Ok().finish()
}