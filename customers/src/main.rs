use std::{env, sync::Arc};

use actix_web::HttpServer;
use common::{
    context::ServiceState,
    entities::{customer::Customer, project::Project},
    repository::mongo_repository::MongoRepository,
};

use customers::create_app;
use mongodb::bson::oid::ObjectId;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();

    env_logger::init();

    let customer_repo: MongoRepository<Customer<ObjectId>> =
        MongoRepository::new(&mongo_uri, "customers", "customers").await;
    let project_repo: MongoRepository<Project<ObjectId>> =
        MongoRepository::new(&mongo_uri, "customers", "projects").await;

    let mut state = ServiceState::new("customer".to_string());
    state.insert(Arc::new(customer_repo));
    state.insert(Arc::new(project_repo));
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3002))?
        .run()
        .await
}
