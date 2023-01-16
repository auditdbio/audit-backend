#[macro_use]
extern crate lazy_static;

mod error;
mod handlers;
mod repositories;
mod utils;
mod ruleset;
pub mod constants;

use std::env;

use actix_web::{middleware, HttpServer, App, web};
use handlers::{user::{post_user, patch_user, delete_user, get_users, get_user}, auth::{login, restore, verify}};
use repositories::{user::UserRepository, token::TokenRepository};
pub use utils::prelude;
use utoipa::OpenApi;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();
    env_logger::init();

    let user_repo = UserRepository::new(mongo_uri.clone()).await;
    let token_repo = TokenRepository::new(mongo_uri.clone()).await;
    HttpServer::new(move || {
        App::new()
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
    .bind(("127.0.0.1", 3001))?
    .run()
    .await
}
