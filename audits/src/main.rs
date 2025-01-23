use std::{env, sync::Arc};
use actix_web::HttpServer;
use mongodb::bson::oid::ObjectId;

use common::{
    auth::Service,
    context::effectfull_context::ServiceState,
    entities::{audit::Audit, audit_request::AuditRequest},
    repository::mongo_repository::MongoRepository,
    verification::verify,
};
use audits::{create_app, migrations::up_migrations};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    up_migrations(mongo_uri.as_str()).await.expect("Migration error");

    verify::<Audit<ObjectId>>(mongo_uri.as_str(), "audits", "audits", true)
        .await
        .expect("Audits collection verification fail");

    // verify::<AuditRequest<ObjectId>>(mongo_uri.as_str(), "audits", "requests", true)
    //     .await
    //     .expect("Audit requests collection verification fail");

    let audit_repo: MongoRepository<Audit<ObjectId>> =
        MongoRepository::new(&mongo_uri, "audits", "audits").await;
    let audit_request_repo: MongoRepository<AuditRequest<ObjectId>> =
        MongoRepository::new(&mongo_uri, "audits", "requests").await;

    let mut state = ServiceState::new(Service::Audits);
    state.insert(Arc::new(audit_repo));
    state.insert(Arc::new(audit_request_repo));
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3003))?
        .run()
        .await
}
