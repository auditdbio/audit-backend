mod constants;
mod error;
mod handlers;
mod repositories;
mod ruleset;
mod utils;

use std::env;

use actix_web::middleware;
use actix_web::web;
pub use utils::prelude;

pub use handlers::auth::*;
pub use handlers::user::*;

use crate::repositories::user::UserRepository;
use crate::repositories::token::TokenRepository;

pub async fn configure_service(cfg: &mut web::ServiceConfig) {
    #[cfg(test)]
    let mongo_uri = env::var("MONGOURI").unwrap();

    #[cfg(not(test))]
    let mongo_uri = env::var("MONGOURI_TEST").unwrap();
   
    let user_repo = UserRepository::new(mongo_uri.clone()).await;
    let token_repo = TokenRepository::new(mongo_uri.clone()).await;

    cfg
        .app_data(web::Data::new(user_repo.clone()))
        .app_data(web::Data::new(token_repo.clone()))
        .service(post_user)
        .service(patch_user)
        .service(delete_user)
        .service(get_users)
        .service(get_user)
        .service(login)
        .service(restore)
        .service(verify);
}
