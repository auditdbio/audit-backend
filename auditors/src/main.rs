use std::env;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use auditors::test_query;
use handlers::auditor::{delete_auditor, get_auditor, get_auditors, patch_auditor, post_auditor};
use repositories::auditor::AuditorRepository;

mod error;
pub mod handlers;
mod repositories;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();

    env_logger::init();

    let auditor_repo = web::Data::new(AuditorRepository::new(mongo_uri.clone()).await);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(auditor_repo.clone())
            .service(post_auditor)
            .service(get_auditor)
            .service(patch_auditor)
            .service(delete_auditor)
            .service(test_query)
            .service(get_auditors)
    })
    .bind(("0.0.0.0", 3004))?
    .run()
    .await
}
