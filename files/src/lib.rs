use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};
use common::{context::effectfull_context::ServiceState, services::API_PREFIX};
pub use handlers::*;
use handlers::{
    file::*,
    indexer::ping,
};
use crate::file::{get_file_by_id, get_meta_by_id};

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
        .app_data(web::Data::new(state))
        .service(
            web::scope(&API_PREFIX)
                .service(create_file)
                .service(find_file)
                .service(get_file_by_id)
                .service(get_meta_by_id)
                .service(delete_file)
                .service(delete_file_by_id)
                .service(change_file_meta)
                .service(change_file_meta_by_id)
                .service(ping),
        );

    app
}
