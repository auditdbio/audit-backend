use actix_web::rt::{spawn, time};
use common::context::ServiceState;
use common::repository::mongo_repository::MongoRepository;
use common::repository::Repository;
use common::services::{CUSTOMERS_SERVICE, AUDITORS_SERVICE};
use log::info;
use mongodb::bson::Bson;
use search::create_app;
use search::repositories::search::SearchRepo;
use search::repositories::since::Since;
use search::service::search::fetch_data;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();
    let search_repo = SearchRepo::new(mongo_uri.clone()).await;

    let since_repo = Arc::new(MongoRepository::new(&mongo_uri, "search", "meta").await);
    if since_repo
        .find("name", &Bson::String("since".to_string()))
        .await
        .unwrap()
        .is_none()
    {
        since_repo.insert(&Since::default()).await.unwrap();
    }

    let timeout = env::var("TIMEOUT")
        .unwrap_or("7200".to_string())
        .parse::<u64>()
        .unwrap();

    let search_repo_clone = search_repo.clone();
    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(timeout));
        loop {
            info!("start waiting");
            interval.tick().await;
            info!("end waiting");

            fetch_data(since_repo.clone(), search_repo_clone.clone()).await;
        }
    });

    let state = Arc::new(ServiceState::new("search".to_string()));

    log::info!("{} {}", CUSTOMERS_SERVICE.as_str(), AUDITORS_SERVICE.as_str());

    HttpServer::new(move || create_app(state.clone(), search_repo.clone()))
        .bind(("0.0.0.0", 3006))?
        .run()
        .await
}
