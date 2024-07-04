use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};

use common::{
    context::effectfull_context::ServiceState,
    services::API_PREFIX,
};
pub use handlers::rating::*;

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

    App::new()
        .wrap(cors)
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(state))
        .service(
            web::scope(&API_PREFIX)
                .service(get_user_rating)
                .service(get_user_rating_details)
                .service(recalculate_rating)
                .service(send_feedback)
                .service(get_feedback)
        )
}
