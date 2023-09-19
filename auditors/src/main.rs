use std::env;
use std::sync::Arc;

use actix_web::HttpServer;
use auditors::create_app;

use common::context::ServiceState;
use common::entities::auditor::Auditor;
use common::entities::bage::Bage;
use common::repository::mongo_repository::MongoRepository;
use mongodb::bson::oid::ObjectId;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let mongo_uri = env::var("MONGOURI").unwrap();
    env_logger::init();

    let auditor_repo: MongoRepository<Auditor<ObjectId>> =
        MongoRepository::new(&mongo_uri, "auditors", "auditors").await;
    let bage_repo: MongoRepository<Bage<ObjectId>> =
        MongoRepository::new(&mongo_uri, "bages", "bages").await;

    let mut state = ServiceState::new("customer".to_string());
    state.insert(Arc::new(auditor_repo));
    state.insert(Arc::new(bage_repo));
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3004))?
        .run()
        .await
}
