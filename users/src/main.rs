extern crate lazy_static;

use users::*;
use users::repositories::{token::TokenRepository, user::UserRepository};

use std::env;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = env::var("MONGOURI").unwrap();

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
            .service(get_user_email)
            .service(login)
            .service(restore)
            .service(verify)
    })
    .bind(("0.0.0.0", 3001))?
    .run()
    .await
}
