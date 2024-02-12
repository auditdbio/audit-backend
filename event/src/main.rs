use std::sync::Arc;

use actix_web::HttpServer;
use common::auth::Service;
use common::context::effectfull_context::ServiceState;
use event::{create_app, service::event::SessionManager};
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let state = ServiceState::new(Service::Event);

    let state = Arc::new(state);

    let manager = Arc::new(Mutex::new(SessionManager::default()));

    HttpServer::new(move || create_app(state.clone(), manager.clone()))
        .bind(("0.0.0.0", 3010))?
        .run()
        .await
}
