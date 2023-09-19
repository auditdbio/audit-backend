extern crate lazy_static;

use common::context::ServiceState;

use telemetry::create_app;

use std::sync::Arc;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let state = ServiceState::new("telemetry".to_string());

    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3009))?
        .run()
        .await
}
