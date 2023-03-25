use common::auth_session::{AuthSessionManager, HttpSessionManager};
use search::repositories::search::SearchRepo;
use search::create_app;

use std::env;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();
    let search_repo = SearchRepo::new(mongo_uri).await;
    let manager = AuthSessionManager::new(HttpSessionManager);

    HttpServer::new(move || create_app(manager.clone(), search_repo.clone()))
        .bind(("0.0.0.0", 3001))?
        .run()
        .await
}
