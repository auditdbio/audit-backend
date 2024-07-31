pub mod handlers;
pub mod service;

use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{
    App, middleware, web,
    body::MessageBody,
    dev::{ServiceFactory, ServiceResponse, ServiceRequest}
};

use common::{
    context::effectfull_context::ServiceState,
    services::API_PREFIX
};

pub use handlers::auth::*;
pub use handlers::user::*;
pub use handlers::organization::*;

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
                .service(change_user)
                .service(delete_user)
                .service(find_user)
                .service(login)
                .service(my_user)
                .service(verify_link)
                .service(create_user)
                .service(forgot_password)
                .service(reset_password)
                .service(restore_token)
                .service(github_auth)
                .service(add_linked_account)
                .service(patch_linked_account)
                .service(delete_linked_account)
                .service(add_wallet)
                .service(find_user_by_email)
                .service(proxy_github_api)
                .service(proxy_github_files)
                .service(find_user_by_email)
                .service(create_organization)
                .service(get_organization)
                .service(get_my_organizations)
                .service(add_members)
                .service(delete_member)
                .service(change_organization)
                .service(change_access)
                .service(add_organization_linked_account)
                .service(delete_organization_linked_account)
                .service(get_invites)
                .service(confirm_invite)
                .service(cancel_invite)
        );
    app
}
