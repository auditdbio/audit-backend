use std::{env, sync::Arc};
use mongodb::bson::oid::ObjectId;
use actix_web::HttpServer;

use common::{
    auth::Service,
    context::effectfull_context::ServiceState,
    entities::{customer::Customer, project::Project},
    repository::mongo_repository::MongoRepository,
    verification::verify,
};
use customers::{create_app, migrations::up_migrations};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    up_migrations(mongo_uri.as_str()).await.expect("Migration error");
    verify::<Project<ObjectId>>(mongo_uri.as_str(), "customers", "projects", true)
        .await
        .expect("Projects collection verification fail");

    let customer_repo: MongoRepository<Customer<ObjectId>> =
        MongoRepository::new(&mongo_uri, "customers", "customers").await;
    let project_repo: MongoRepository<Project<ObjectId>> =
        MongoRepository::new(&mongo_uri, "customers", "projects").await;

    let mut state = ServiceState::new(Service::Customers);
    state.insert(Arc::new(customer_repo));
    state.insert(Arc::new(project_repo));
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3002))?
        .run()
        .await
}
