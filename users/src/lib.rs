mod constants;
mod error;
mod handlers;
pub mod repositories;
mod ruleset;
mod utils;

use std::env;

use actix_cors::Cors;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceFactory;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::middleware;
use actix_web::web;
use actix_web::App;
use common::repository::test_repository::TestRepository;
use repositories::token::TokenRepo;
use repositories::user::UserRepo;
pub use utils::prelude;

pub use handlers::auth::*;
pub use handlers::user::*;

pub fn create_app(
    user_repo: UserRepo,
    token_repo: TokenRepo,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let cors = Cors::default()
        .allow_any_origin()
        .allow_any_header()
        .allow_any_method();
    let app = App::new()
        .wrap(cors)
        .wrap(middleware::Logger::default())
        .app_data(web::Data::new(user_repo))
        .app_data(web::Data::new(token_repo))
        .service(post_user)
        .service(patch_user)
        .service(delete_user)
        .service(get_users)
        .service(get_user)
        .service(get_user_email)
        .service(login)
        .service(restore)
        .service(verify);
    app
}

pub fn create_test_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let user_repo = UserRepo::new(TestRepository::new());

    let token_repo = TokenRepo::new(TestRepository::new());

    create_app(user_repo, token_repo)
}
