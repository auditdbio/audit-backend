use common::auth_session::{AuthSessionManager, HttpSessionManager};
use search::create_app;

use std::env;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let manager = AuthSessionManager::new(HttpSessionManager);

    HttpServer::new(move || create_app(manager.clone()))
        .bind(("0.0.0.0", 3001))?
        .run()
        .await
}
