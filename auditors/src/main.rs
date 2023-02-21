use std::env;

use actix_web::{middleware, web, App, HttpServer};
use auditors::create_app;
use auditors::repositories::auditor::AuditorRepo;
use common::repository::mongo_repository::MongoRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();
    env_logger::init();

    let auditor_repo =
        AuditorRepo::new(MongoRepository::new(&mongo_uri, "auditors", "auditors").await);
    HttpServer::new(move || create_app(auditor_repo.clone()))
        .bind(("0.0.0.0", 3004))?
        .run()
        .await
}
