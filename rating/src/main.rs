use actix_web::HttpServer;
use std::{env, sync::Arc};
use mongodb::bson::oid::ObjectId;

use rating::create_app;
use common::{
    auth::Service,
    context::effectfull_context::ServiceState,
    entities::rating::Rating,
    repository::mongo_repository::MongoRepository,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let mut state = ServiceState::new(Service::Rating);
    let mongo_uri = env::var("MONGOURI").unwrap();

    let ratings_repo:MongoRepository<Rating<ObjectId>> = MongoRepository::new(&mongo_uri, "ratings", "ratings").await;

    state.insert(Arc::new(ratings_repo));
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3014))?
        .run()
        .await
}
