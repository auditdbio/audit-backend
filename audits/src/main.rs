use std::env;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use audits::{get_audits, post_audit_request, patch_audit_request, delete_audit_request, post_audit, delete_audit, get_views};
use audits::repositories::{audit::AuditRepo, audit_request::AuditRequestRepo};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let audit_repo = AuditRepo::new(mongo_uri.clone()).await;
    let audit_request_repo = AuditRequestRepo::new(mongo_uri.clone()).await;
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(audit_repo.clone()))
            .app_data(web::Data::new(audit_request_repo.clone()))
            .service(get_audits)
            .service(post_audit_request)
            .service(patch_audit_request)
            .service(delete_audit_request)
            .service(post_audit)
            .service(get_audits)
            .service(delete_audit)
            .service(get_views)
    })
    .bind(("0.0.0.0", 3003))?
    .run()
    .await
}
