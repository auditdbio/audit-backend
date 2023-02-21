pub mod error;
pub mod handlers;
pub mod repositories;
pub mod ruleset;

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};
use common::repository::test_repository::TestRepository;
pub use handlers::auditor::*;
use repositories::auditor::AuditorRepo;

pub fn create_app(
    auditor_repo: AuditorRepo,
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
        .app_data(web::Data::new(auditor_repo))
        .service(post_auditor)
        .service(get_auditor)
        .service(patch_auditor)
        .service(delete_auditor)
        .service(get_auditors);
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
    let auditor_repo = AuditorRepo::new(TestRepository::new());

    create_app(auditor_repo)
}
