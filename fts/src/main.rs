use std::path::PathBuf;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};

mod api;
mod db;
mod error;
mod index;
mod models;
mod search;
mod storage;

use error::ServiceResult;
use search::SearchService;

#[actix_web::main]
async fn main() -> ServiceResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get configuration from environment
    let mongo_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "auditors".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    // Create data directories
    let data_dir = PathBuf::from("data");
    std::fs::create_dir_all(&data_dir)?;

    let index_path = data_dir.join("index");
    let storage_path = data_dir.join("storage");

    // Initialize search service
    let service = SearchService::new(index_path, storage_path, &mongo_uri, &db_name).await?;
    let service = Data::new(service);

    // Start HTTP server
    tracing::info!("Starting server at http://{}:{}", host, port);

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
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}
