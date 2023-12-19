use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};
use common::context::effectfull_context::ServiceState;

pub mod handlers;
pub mod repositories;
pub mod services;

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
        .service(handlers::chat::messages)
        .service(handlers::chat::preview)
        .service(handlers::chat::send_message)
        .service(handlers::chat::chat_unread);
    app
}
