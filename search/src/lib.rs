pub mod handlers;

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
pub use handlers::search::*;

pub fn create_app(
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
        .app_data(web::Data::new(manager))
        .service(search);
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
    let test_manager = AuthSessionManager::new(TestSessionManager(user));

    create_app(test_manager)
}
