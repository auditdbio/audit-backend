use std::sync::Arc;

use actix_web::HttpServer;
use chat::create_app;
use common::context::ServiceState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let state = ServiceState::new("chat".to_string());

    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3012))?
        .run()
        .await
}
