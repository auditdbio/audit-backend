extern crate lazy_static;

use common::auth_session::{AuthSessionManager, HttpSessionManager};
use common::repository::mongo_repository::MongoRepository;
use users::repositories::list_element::ListElementRepository;
use users::repositories::{token::TokenRepo, user::UserRepo};
use users::*;

use std::env;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let user_repo = UserRepo::new(MongoRepository::new(&mongo_uri, "users", "users").await);
    let token_repo = TokenRepo::new(MongoRepository::new(&mongo_uri, "users", "tokens").await);
    let elem_repo = ListElementRepository::new(MongoRepository::new(&mongo_uri, "users", "list_elements").await);
    let manager = AuthSessionManager::new(HttpSessionManager);

    HttpServer::new(move || create_app(user_repo.clone(), token_repo.clone(), elem_repo.clone(), manager.clone()))
        .bind(("0.0.0.0", 3001))?
        .run()
        .await
}
