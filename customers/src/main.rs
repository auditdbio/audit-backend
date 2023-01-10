pub mod handlers;
mod repositories;
mod error;

use std::env;

use actix_web::{middleware, HttpServer, App, web};
use handlers::{customers::{post_customer, patch_customer, delete_customer, get_customer}, projects::{get_project, post_project, delete_project, patch_project}};
use repositories::{customer::CustomerRepository, project::ProjectRepository};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();
    env_logger::init();

    let customer_repo = CustomerRepository::new(mongo_uri.clone()).await;
    let project_repo = ProjectRepository::new(mongo_uri.clone()).await;
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(customer_repo.clone()))
            .app_data(web::Data::new(project_repo.clone()))
            .service(post_customer)
            .service(get_customer)
            .service(patch_customer)
            .service(delete_customer)

            .service(post_project)
            .service(get_project)
            .service(patch_project)
            .service(delete_project)
    })
    .bind(("127.0.0.1", 3001))?
    .run()
    .await
}
