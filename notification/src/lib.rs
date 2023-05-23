use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};
use common::context::ServiceState;
use handlers::notifications::{notifications, read_notification, send_notification};
use repositories::notifications::NotificationsRepository;

pub mod access_rules;
pub mod handlers;
pub mod repositories;
pub mod service;

pub fn create_app(
    state: Arc<ServiceState>,
    repo: Arc<NotificationsRepository>,
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
        .app_data(web::Data::from(repo))
        .service(send_notification)
        .service(notifications)
        .service(read_notification);
    app
}
