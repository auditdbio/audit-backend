pub mod handlers;
pub mod service;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};
use common::context::ServiceState;
pub use handlers::auditor::*;
use handlers::indexer::{get_auditor_data, ping, provide_auditor_data};

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
        .service(post_auditor)
        .service(get_auditor)
        .service(patch_auditor)
        .service(delete_auditor)
        .service(provide_auditor_data)
        .service(get_my_auditor)
        .service(get_auditor_data)
        .service(ping);
    app
}
