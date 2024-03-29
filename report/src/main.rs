use std::sync::Arc;

use actix_web::HttpServer;
use common::auth::Service;
use common::context::effectfull_context::ServiceState;
use report::create_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let state = Arc::new(ServiceState::new(Service::Report));

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3011))?
        .run()
        .await
}
