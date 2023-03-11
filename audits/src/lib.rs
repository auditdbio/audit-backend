pub mod contants;
pub mod error;
pub mod handlers;
pub mod repositories;
pub mod ruleset;

use actix_cors::Cors;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::middleware;
use actix_web::web;
use actix_web::App;
use common::auth_session::AuthSession;
use common::auth_session::AuthSessionManager;

use common::auth_session::TestSessionManager;
use common::repository::test_repository::TestRepository;
pub use handlers::audit::*;
pub use handlers::audit_request::*;
use repositories::audit::AuditRepo;
use repositories::audit_request::AuditRequestRepo;
use repositories::closed_audits::ClosedAuditRepo;
use repositories::closed_request::ClosedAuditRequestRepo;

pub fn create_app(
    audit_repo: AuditRepo,
    audit_request_repo: AuditRequestRepo,
    closed_audit_repo: ClosedAuditRepo,
    closed_audit_request_repo: ClosedAuditRequestRepo,
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
        .app_data(web::Data::new(audit_repo))
        .app_data(web::Data::new(audit_request_repo))
        .app_data(web::Data::new(closed_audit_repo))
        .app_data(web::Data::new(closed_audit_request_repo))
        .app_data(web::Data::new(manager))
        .service(post_audit_request)
        .service(get_audit_requests)
        .service(patch_audit_request)
        .service(delete_audit_request)
        .service(post_audit)
        .service(delete_audit)
        .service(get_audit)
        .service(get_audits)
        .service(get_views)
        .service(patch_audit)
        .service(requests_by_id)
        .service(audit_by_id);
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
    let audit_repo = AuditRepo::new(TestRepository::new());
    let closed_audit_repo = ClosedAuditRepo::new(TestRepository::new());
    let audit_request_repo = AuditRequestRepo::new(TestRepository::new());
    let closed_audit_request_repo = ClosedAuditRequestRepo::new(TestRepository::new());
    let test_manager = AuthSessionManager::new(TestSessionManager(user));

    create_app(
        audit_repo,
        audit_request_repo,
        closed_audit_repo,
        closed_audit_request_repo,
        test_manager,
    )
}
