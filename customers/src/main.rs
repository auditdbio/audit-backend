mod error;
pub mod handlers;
mod repositories;

use std::env;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use customers::{get_projects, test_query};
use handlers::{
    customers::{delete_customer, get_customer, patch_customer, post_customer},
    projects::{delete_project, get_project, patch_project, post_project},
};
use repositories::{customer::CustomerRepository, project::ProjectRepository};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();

    env_logger::init();

    let customer_repo = CustomerRepository::new(mongo_uri.clone()).await;
    let project_repo = ProjectRepository::new(mongo_uri.clone()).await;
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        App::new()
            .wrap(cors)
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
            .service(get_projects)
            .service(test_query)
    })
    .bind(("0.0.0.0", 3002))?
    .run()
    .await
}
