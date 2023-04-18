use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, middleware, web, dev::{ServiceFactory, ServiceResponse, ServiceRequest}, body::MessageBody};
use common::context::ServiceState;
use handlers::mail::sent_mail;

pub mod handlers;
pub mod service;

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
    let app = App::new()
        .wrap(cors)
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(state))
        .service(sent_mail);
    app
}
