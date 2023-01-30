pub mod contants;
pub mod error;
pub mod handlers;
pub mod repositories;
pub mod ruleset;

use std::env;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use repositories::{audit::AuditRepo, audit_request::AuditRequestRepo};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(test)]
    let mongo_uri = env::var("MONGOURI").unwrap();

    #[cfg(not(test))]
    let mongo_uri = env::var("MONGOURI_TEST").unwrap();
    env_logger::init();

    let user_repo = AuditRepo::new(mongo_uri.clone()).await;
    let token_repo = AuditRequestRepo::new(mongo_uri.clone()).await;
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(user_repo.clone()))
            .app_data(web::Data::new(token_repo.clone()))
    })
    .bind(("0.0.0.0", 3003))?
    .run()
    .await
}
