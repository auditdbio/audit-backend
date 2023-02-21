pub mod error;
pub mod handlers;
pub mod repositories;

use actix_cors::Cors;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::middleware;
use actix_web::web;
use actix_web::App;
use common::repository::test_repository::TestRepository;
use repositories::customer::CustomerRepo;
use repositories::project::ProjectRepo;

pub use crate::handlers::customers::*;
pub use crate::handlers::projects::*;

pub fn create_app(
    customer_repo: CustomerRepo,
    project_repo: ProjectRepo,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let cors = Cors::default()
        .allow_any_origin()
        .allow_any_header()
        .allow_any_method();
    let app = App::new()
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
        .service(get_projects);
    app
}

pub fn create_test_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let user_repo = CustomerRepo::new(TestRepository::new());

    let token_repo = ProjectRepo::new(TestRepository::new());

    create_app(user_repo, token_repo)
}
