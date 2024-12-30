use std::{env, sync::Arc};

use actix_web::HttpServer;
use common::{
    auth::Service,
    context::effectfull_context::ServiceState,
    repository::mongo_repository::MongoRepository,
};
use notification::{
    create_app, repositories::notifications::NotificationsRepository, migrations::up_migrations,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").expect("MONGOURI must be set");

    up_migrations(mongo_uri.as_str()).await.expect("Migration error");

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
