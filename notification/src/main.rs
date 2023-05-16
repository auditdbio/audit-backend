use std::{env, sync::Arc};

use actix_web::HttpServer;
use common::{context::ServiceState, repository::mongo_repository::MongoRepository};
use notification::{
    create_app, repositories::notifications::NotificationsRepository,
    service::notifications::NotificationsManager,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").expect("MONGOURI must be set");

    let state = ServiceState::new("notification".to_string());

    let state = Arc::new(state);

    let manager = Arc::new(NotificationsManager::new());

    let repo = Arc::new(NotificationsRepository::new(
        MongoRepository::new(&mongo_uri, "notification", "notifications").await,
    ));

    HttpServer::new(move || create_app(state.clone(), repo.clone(), manager.clone()))
        .bind(("0.0.0.0", 3008))?
        .run()
        .await
}
