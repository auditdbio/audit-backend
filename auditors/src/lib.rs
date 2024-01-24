pub mod handlers;
pub mod service;

use std::{sync::Arc};

use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware,
    web::{self},
    App,
};
use common::{context::effectfull_context::ServiceState, services::API_PREFIX};
pub use handlers::auditor::*;
use handlers::{
    badge::{delete, find_badge, merge, post_badge},
    indexer::{get_auditor_data, get_badges_data, ping, provide_auditor_data, provide_badges_data},
};

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
                .service(post_auditor)
                .service(get_auditor)
                .service(patch_auditor)
                .service(delete_auditor)
                .service(provide_auditor_data)
                .service(provide_badges_data)
                .service(get_my_auditor)
                .service(get_auditor_data)
                .service(get_badges_data)
                .service(ping)
                .service(post_badge)
                .service(merge)
                .service(delete)
                .service(find_badge),
        );
    app
}
