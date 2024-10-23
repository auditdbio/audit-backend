use std::{env, sync::Arc};
use actix_web::HttpServer;

use common::{
    auth::Service,
    context::effectfull_context::ServiceState,
    entities::file::Metadata,
    repository::mongo_repository::MongoRepository,
};
use files::create_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let meta_repo: MongoRepository<Metadata> =
        MongoRepository::new(&mongo_uri, "files", "meta").await;

    let mut state = ServiceState::new(Service::Files);
    state.insert(Arc::new(meta_repo));
    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3005))?
        .run()
        .await
}
