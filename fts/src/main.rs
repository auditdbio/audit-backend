use std::{env, time::Duration};

use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use tokio::time;
use dotenv::dotenv;

mod api;
mod db;
mod error;
mod index;
mod models;
mod search;
mod storage;

use error::ServiceResult;
use search::SearchService;

async fn sync_task(service: Data<SearchService>) {
    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        tracing::info!("Starting periodic sync...");
        match service.sync().await {
            Ok(_) => tracing::info!("Periodic sync completed successfully"),
            Err(e) => tracing::error!("Periodic sync failed: {:?}", e),
        }
    }
}

#[actix_web::main]
async fn main() -> ServiceResult<()> {
    dotenv().ok();
    // Initialize logging with debug level
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Get configuration from environment
    let mongo_uri = env::var("MONGOURI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    // Create data directories
    let current_dir = env::current_dir().unwrap();
    tracing::info!("Current working directory: {:?}", current_dir);
    let data_dir = current_dir.join("data");
    std::fs::create_dir_all(&data_dir)?;

    let index_path = data_dir.join("index");
    std::fs::create_dir_all(&index_path)?;
    let storage_path = data_dir.join("storage");

    // Initialize search service
    let service = SearchService::new(index_path, storage_path, &mongo_uri).await?;
    let service = Data::new(service);

    let service_for_task = service.clone();

    tokio::spawn(async move {
        sync_task(service_for_task).await;
    });

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(service.clone())
            .configure(api::configure)
    })
    .bind(("0.0.0.0", 3020))?
    .run()
    .await?;

    Ok(())
}
