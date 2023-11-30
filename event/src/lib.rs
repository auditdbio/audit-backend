use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};
use common::context::effectfull_context::ServiceState;
use service::event::SessionManager;
use tokio::sync::Mutex;

pub mod handlers;
pub mod service;

pub fn create_app(
    state: Arc<ServiceState>,
    manager: Arc<Mutex<SessionManager>>,
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
        .app_data(web::Data::from(manager))
        .app_data(web::Data::new(state))
        .service(handlers::event::events)
        .service(handlers::event::make_event)
        .service(handlers::ping);
    app
}
