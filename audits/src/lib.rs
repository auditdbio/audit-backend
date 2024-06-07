pub mod handlers;
pub mod migrations;
pub mod service;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::middleware;
use actix_web::web;
use actix_web::App;

use common::context::effectfull_context::ServiceState;
use common::services::API_PREFIX;
pub use handlers::audit::*;
pub use handlers::audit_request::*;

#[must_use]
pub fn create_app(
    state: Arc<ServiceState>,
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

    #[allow(clippy::let_and_return)]
    let app = App::new()
        .wrap(cors)
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(state))
        .service(
            web::scope(&API_PREFIX)
                .service(post_audit)
                .service(get_audit)
                .service(patch_audit)
                .service(delete_audit)
                .service(post_audit_request)
                .service(get_audit_request)
                .service(patch_audit_request)
                .service(delete_audit_request)
                .service(get_request_edit_history)
                .service(change_audit_edit_history)
                .service(get_my_audit)
                .service(get_my_audit_request)
                .service(post_audit_issue)
                .service(patch_audit_issue)
                .service(get_audit_issue)
                .service(get_audit_issue_by_id)
                .service(delete_audit_issue)
                .service(patch_audit_disclose_all)
                .service(patch_audit_issue_read)
                .service(get_public_audits)
                .service(find_all_audit_request)
                .service(post_no_customer_audit)
                .service(get_audit_edit_history)
                .service(audit_unread_edits)
                .service(request_unread_edits),
        );

    app
}
