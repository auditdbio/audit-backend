use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};
use common::context::ServiceState;
use handlers::notifications::{notifications, send_notification};

pub mod access_rules;
pub mod handlers;
pub mod repositories;
pub mod service;

pub fn create_app(
    state: Arc<ServiceState>,
    manager: Arc<service::notifications::NotificationsManager>,
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
        .app_data(web::Data::from(manager))
        .app_data(web::Data::new(state))
        .service(send_notification)
        .service(notifications);
    app
}
