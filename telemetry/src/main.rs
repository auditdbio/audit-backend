extern crate lazy_static;

use common::context::ServiceState;
use common::entities::user::User;
use common::repository::mongo_repository::MongoRepository;
use mongodb::bson::oid::ObjectId;
use telemetry::create_app;

use std::env;
use std::sync::Arc;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let state = ServiceState::new("telemetry".to_string());

    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3009))?
        .run()
        .await
}
