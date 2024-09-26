use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware,
    web::{self, Data},
    App,
};
use common::{
    context::effectfull_context::ServiceState,
    services::API_PREFIX,
};

use crate::handlers::report::*;

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

    #[allow(clippy::let_and_return)]
    let app = App::new()
        .wrap(cors)
        .wrap(middleware::Logger::default())
        .app_data(Data::new(state))
        .service(
            web::scope(&API_PREFIX)
                .service(create_report)
                .service(verify_report)
        );
    app
}
