use std::env;


use actix_web::{HttpServer};
use audits::repositories::closed_audits::ClosedAuditRepo;
use audits::repositories::closed_request::ClosedAuditRequestRepo;
use audits::repositories::{audit::AuditRepo, audit_request::AuditRequestRepo};
use audits::{
    create_app,
};
use common::auth_session::{AuthSessionManager, HttpSessionManager};
use common::repository::mongo_repository::MongoRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let audit_repo = AuditRepo::new(MongoRepository::new(&mongo_uri, "audits", "audits").await);
    let audit_request_repo =
        AuditRequestRepo::new(MongoRepository::new(&mongo_uri, "audits", "requests").await);
    let closed_audit_repo =
        ClosedAuditRepo::new(MongoRepository::new(&mongo_uri, "audits", "closed_audits").await);
    let closed_audit_request_repo = ClosedAuditRequestRepo::new(
        MongoRepository::new(&mongo_uri, "audits", "closed_requests").await,
    );
    let manager = AuthSessionManager::new(HttpSessionManager);

    HttpServer::new(move || {
        create_app(
            audit_repo.clone(),
            audit_request_repo.clone(),
            closed_audit_repo.clone(),
            closed_audit_request_repo.clone(),
            manager.clone(),
        )
    })
    .bind(("0.0.0.0", 3003))?
    .run()
    .await
}
