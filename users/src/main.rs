extern crate lazy_static;

pub mod constants;
mod error;
mod handlers;
mod repositories;
mod ruleset;
mod utils;

use std::env;

use actix_cors::Cors;
use actix_web::{http, middleware, web, App, HttpServer};
use handlers::{
    auth::{login, restore, verify},
    user::{delete_user, get_user, get_users, patch_user, post_user},
};
use repositories::{token::TokenRepository, user::UserRepository};
pub use utils::prelude;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(test)]
    let mongo_uri = env::var("MONGOURI").unwrap();
    #[cfg(not(test))]
    let mongo_uri = env::var("MONGOURI_TEST").unwrap();

    env_logger::init();

    let user_repo = UserRepository::new(mongo_uri.clone()).await;
    let token_repo = TokenRepository::new(mongo_uri.clone()).await;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(user_repo.clone()))
            .app_data(web::Data::new(token_repo.clone()))
            .service(post_user)
            .service(patch_user)
            .service(delete_user)
            .service(get_users)
            .service(get_user)
            .service(login)
            .service(restore)
            .service(verify)
    })
    .bind(("0.0.0.0", 3001))?
    .run()
    .await
}
