pub mod contants;
pub mod error;
pub mod handlers;
pub mod repositories;
pub mod ruleset;

pub use handlers::audit::*;
pub use handlers::audit_request::*;
use repositories::audit::AuditRepo;
use repositories::audit_request::AuditRequestRepo;
use repositories::closed_audits::ClosedAuditRepo;
use repositories::closed_request::ClosedRequestRepo;

pub fn create_app(
    audit_repo: AuditRepo,
    audit_request_repo: AuditRequestRepo,
    closed_audit_repo: ClosedAuditRepo,
    closed_audit_request_repo: ClosedRequestRepo,

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
