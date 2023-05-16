use std::sync::Arc;

use actix_web::HttpServer;
use common::context::ServiceState;
use notification::{create_app, service::notifications::NotificationsManager};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let state = ServiceState::new("notification".to_string());

    let state = Arc::new(state);

    let manager = Arc::new(NotificationsManager::new());

    HttpServer::new(move || create_app(state.clone(), manager.clone()))
        .bind(("0.0.0.0", 3008))?
        .run()
        .await
}
