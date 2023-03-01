use std::env;


use actix_web::{HttpServer};
use common::{
    auth_session::{AuthSessionManager, HttpSessionManager},
    repository::mongo_repository::MongoRepository,
};

use customers::repositories::{customer::CustomerRepo, project::ProjectRepo};
use customers::{create_app};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();

    env_logger::init();

    let customer_repo =
        CustomerRepo::new(MongoRepository::new(&mongo_uri, "customers", "customers").await);
    let project_repo =
        ProjectRepo::new(MongoRepository::new(&mongo_uri, "customers", "projects").await);
    let manager = AuthSessionManager::new(HttpSessionManager);
    HttpServer::new(move || {
        create_app(customer_repo.clone(), project_repo.clone(), manager.clone())
    })
    .bind(("0.0.0.0", 3002))?
    .run()
    .await
}
