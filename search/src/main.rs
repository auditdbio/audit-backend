use actix_web::rt::{spawn, time};
use common::auth_session::{AuthSessionManager, HttpSessionManager};
use search::repositories::search::SearchRepo;
use search::repositories::since::SinceRepo;
use search::{create_app, fetch_data};

use std::env;
use std::time::Duration;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();
    let search_repo = SearchRepo::new(mongo_uri.clone()).await;
    let manager = AuthSessionManager::new(HttpSessionManager);
    let since_repo = SinceRepo::new(mongo_uri.clone()).await;

    since_repo.insert_default().await;

    let search_repo_clone = search_repo.clone();
    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(100));
        loop {
            interval.tick().await;
            fetch_data(since_repo.clone(), search_repo_clone.clone()).await;
        }
    });

    HttpServer::new(move || create_app(manager.clone(), search_repo.clone()))
        .bind(("0.0.0.0", 3006))?
        .run()
        .await
}
