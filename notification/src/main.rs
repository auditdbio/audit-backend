use std::{env, sync::Arc};

use actix_web::HttpServer;
use common::repository::mongo_repository::MongoRepository;
use common::{auth::Service, context::effectfull_context::ServiceState};
use notification::{create_app, repositories::notifications::NotificationsRepository};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let mongo_uri = env::var("MONGOURI").expect("MONGOURI must be set");

    let state = ServiceState::new(Service::Notification);

    let state = Arc::new(state);

    let repo = Arc::new(NotificationsRepository::new(
        MongoRepository::new(&mongo_uri, "notification", "notifications").await,
    ));

    HttpServer::new(move || create_app(state.clone(), repo.clone()))
        .bind(("0.0.0.0", 3008))?
        .run()
        .await
}
