use std::{env, sync::Arc};

use actix_web::HttpServer;
use cloc::{create_app, repositories::file_repo::FileRepo};
use common::{
    auth::Service, context::effectfull_context::ServiceState,
    repository::mongo_repository::MongoRepository,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let mongo_uri = env::var("MONGOURI").unwrap();

    env_logger::init();

    let mut state = ServiceState::new(Service::Customers);
    let mongo_repo = MongoRepository::new(&mongo_uri, "cloc", "files").await;
    state.insert_manual(Arc::new(FileRepo::new(
        mongo_repo,
        "/repositories".parse().unwrap(),
    )));
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3013))?
        .run()
        .await
}
