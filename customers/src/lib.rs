pub mod handlers;
pub mod service;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::middleware;
use actix_web::web;
use actix_web::App;


use common::context::ServiceState;

pub use crate::handlers::customer::*;
pub use crate::handlers::project::*;

pub fn create_app(
    context: Arc<ServiceState>,
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
        .app_data(web::Data::new(context))
        .service(post_customer)
        .service(get_customer)
        .service(patch_customer)
        .service(delete_customer)
        .service(post_project)
        .service(get_project)
        .service(patch_project)
        .service(delete_project);
    app
}
