pub mod handlers;
pub mod repositories;
pub mod error;
pub mod ruleset;


use std::env;

use actix_web::{middleware, HttpServer, App, web};
use repositories::{audit::AuditRepo, audit_request::AuditRequestRepo};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();
    env_logger::init();


    let user_repo = AuditRepo::new(mongo_uri.clone()).await;
    let token_repo = AuditRequestRepo::new(mongo_uri.clone()).await;
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(user_repo.clone()))
            .app_data(web::Data::new(token_repo.clone()))

    })
    .bind(("0.0.0.0", 3003))?
    .run()
    .await
}

