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
use common::{
    auth_session::{AuthSession, AuthSessionManager, TestSessionManager},
    repository::test_repository::TestRepository,
};
pub use handlers::auditor::*;
use handlers::get_data;
use repositories::auditor::AuditorRepo;

pub fn create_app(
    auditor_repo: AuditorRepo,
    manager: AuthSessionManager,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let cors = Cors::permissive();
    let app = App::new()
        .wrap(cors)
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(auditor_repo))
        .app_data(web::Data::new(manager))
        .service(post_auditor)
        .service(get_auditor)
        .service(patch_auditor)
        .service(delete_auditor)
        .service(get_auditors)
        .service(auditor_by_id)
        .service(get_data);
    app
}

pub fn create_test_app(
    user: AuthSession,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let auditor_repo = AuditorRepo::new(TestRepository::new());
    let test_manager = AuthSessionManager::new(TestSessionManager(user));
    create_app(auditor_repo, test_manager)
}
