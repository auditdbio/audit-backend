use std::env;

use actix_web::HttpServer;
use common::{repository::mongo_repository::MongoRepository, auth_session::{AuthSessionManager, HttpSessionManager}};
use files::{repositories::{meta::MetadataRepo, files::FilesRepository}, create_app};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let meta_repo = MetadataRepo::new(MongoRepository::new(&mongo_uri, "Users", "users").await);
    let files_repo = FilesRepository {};
    let manager = AuthSessionManager::new(HttpSessionManager);

    HttpServer::new(move || create_app(files_repo, meta_repo.clone(), manager.clone()))
        .bind(("0.0.0.0", 3001))?
        .run()
        .await
}
