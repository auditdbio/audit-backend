use actix_cors::Cors;
use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware, web, App,
};
use common::auth_session::AuthSessionManager;
pub use handlers::*;
use repositories::{files::FilesRepository, meta::MetadataRepo};

pub mod handlers;
pub mod repositories;

pub fn create_app(
    file_repo: FilesRepository,
    meta_repo: MetadataRepo,
    manager: AuthSessionManager,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let cors = Cors::permissive().supports_credentials();
    let app = App::new()
        .wrap(cors)
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(file_repo))
        .app_data(web::Data::new(meta_repo))
        .app_data(web::Data::new(manager))
        .service(create_file)
        .service(get_file);
    app
}
