use std::sync::Arc;

use actix_cors::Cors;
use actix_web::body::MessageBody;
use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    App,
};
use actix_web::{middleware, web};
use common::context::effectfull_context::ServiceState;
use common::services::API_PREFIX;
use handlers::cloc::count;

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
        .service(web::scope(&API_PREFIX).service(count));
    app
}
