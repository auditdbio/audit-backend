use std::env;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use common::repository::mongo_repository::MongoRepository;
use customers::handlers::{
    customers::{delete_customer, get_customer, patch_customer, post_customer},
    projects::{delete_project, get_project, patch_project, post_project},
};
use customers::repositories::{customer::CustomerRepo, project::ProjectRepo};
use customers::{create_app, get_projects};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();

    env_logger::init();

    let customer_repo =
        CustomerRepo::new(MongoRepository::new(&mongo_uri, "customers", "customers").await);
    let project_repo =
        ProjectRepo::new(MongoRepository::new(&mongo_uri, "customers", "projects").await);
    HttpServer::new(move || create_app(customer_repo.clone(), project_repo.clone()))
        .bind(("0.0.0.0", 3002))?
        .run()
        .await
}
