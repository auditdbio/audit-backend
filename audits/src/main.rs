use std::env;
use std::sync::Arc;

use actix_web::HttpServer;
use audits::create_app;
use common::auth_session::{AuthSessionManager, HttpSessionManager};
use common::context::ServiceState;
use common::entities::audit::Audit;
use common::entities::audit_request::AuditRequest;
use common::repository::mongo_repository::MongoRepository;
use mongodb::bson::oid::ObjectId;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let audit_repo: MongoRepository<Audit<ObjectId>> =
        MongoRepository::new(&mongo_uri, "audits", "audits").await;
    let audit_request_repo: MongoRepository<AuditRequest<ObjectId>> =
        MongoRepository::new(&mongo_uri, "audits", "requests").await;

    let mut state = ServiceState::new("audit".to_string());
    state.insert(audit_repo);
    state.insert(audit_request_repo);
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3003))?
        .run()
        .await
}
