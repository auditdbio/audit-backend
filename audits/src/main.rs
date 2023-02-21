use std::env;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use audits::repositories::{audit::AuditRepo, audit_request::AuditRequestRepo};
use audits::{
    delete_audit, delete_audit_request, get_audits, get_views, patch_audit_request, post_audit,
    post_audit_request,
};
use common::repository::mongo_repository::MongoRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let audit_repo = AuditRepo::new(MongoRepository::new(&mongo_uri, "audits", "audits").await);
    let audit_request_repo = AuditRequestRepo::new(MongoRepository::new(&mongo_uri, "audits", "requests").await);
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
