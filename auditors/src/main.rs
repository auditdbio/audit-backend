use std::env;
use std::sync::Arc;

use actix_web::HttpServer;
use auditors::create_app;

use common::auth::Service;
use common::context::effectfull_context::ServiceState;
use common::entities::auditor::Auditor;
use common::entities::badge::Badge;
use common::repository::mongo_repository::MongoRepository;
use mongodb::bson::oid::ObjectId;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let mongo_uri = env::var("MONGOURI").unwrap();
    env_logger::init();

    let auditor_repo: MongoRepository<Auditor<ObjectId>> =
        MongoRepository::new(&mongo_uri, "auditors", "auditors").await;
    let badge_repo: MongoRepository<Badge<ObjectId>> =
        MongoRepository::new(&mongo_uri, "badges", "badges").await;

    let mut state = ServiceState::new(Service::Auditors);
    state.insert(Arc::new(auditor_repo));
    state.insert(Arc::new(badge_repo));
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3004))?
        .run()
        .await
}
