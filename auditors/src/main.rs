use std::env;

use actix_web::{middleware, web, App, HttpServer};
use handlers::auditor::{delete_auditor, get_auditor, get_auditors, patch_auditor, post_auditor};
use repositories::auditor::AuditorRepository;

mod error;
pub mod handlers;
mod repositories;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(test)]
    let mongo_uri = env::var("MONGOURI").unwrap();

    #[cfg(not(test))]
    let mongo_uri = env::var("MONGOURI_TEST").unwrap();
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
