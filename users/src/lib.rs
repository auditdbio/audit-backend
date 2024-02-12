pub mod handlers;
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
pub use handlers::auth::*;
pub use handlers::user::*;

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
                .service(find_user_by_email),
        );
    app
}
