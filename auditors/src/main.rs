use std::env;

use actix_web::{HttpServer, App, middleware, web};
use handlers::auditor::{post_auditor, get_auditor, patch_auditor, delete_auditor, get_auditors};
use repositories::auditor::AuditorRepository;

pub mod handlers;
mod repositories;
mod error;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();
    env_logger::init();

    let auditor_repo = AuditorRepository::new(mongo_uri.clone()).await;
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(auditor_repo.clone()))
            .service(post_auditor)
            .service(get_auditor)
            .service(patch_auditor)
            .service(delete_auditor)
            .service(get_auditors)

    })
    .bind(("0.0.0.0", 3004))?
    .run()
    .await
}

