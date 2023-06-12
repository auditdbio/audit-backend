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
use handlers::indexer::get_customer_data;
use handlers::indexer::get_project_data;
use handlers::indexer::ping;
use handlers::indexer::provide_customer_data;
use handlers::indexer::provide_project_data;

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
    #[allow(clippy::let_and_return)]
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
        .service(delete_project)
        .service(provide_customer_data)
        .service(provide_project_data)
        .service(my_customer)
        .service(my_project)
        .service(get_customer_data)
        .service(get_project_data)
        .service(ping)
        .service(get_customer_projects);
    app
}
