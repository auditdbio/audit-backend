pub mod handlers;
pub mod repositories;
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
use common::context::effectfull_context::ServiceState;
use common::services::API_PREFIX;
pub use handlers::search::*;
use repositories::search::SearchRepo;

pub fn create_app(
    state: Arc<ServiceState>,
    search_repo: SearchRepo,
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
        .app_data(web::Data::new(search_repo))
        .service(
            web::scope(&API_PREFIX)
                .service(insert)
                .service(search)
                .service(mongo_search)
                .service(delete),
        );
    app
}
