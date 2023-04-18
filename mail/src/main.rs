extern crate lazy_static;

use common::context::ServiceState;

use common::repository::mongo_repository::MongoRepository;
use mail::create_app;
use mail::service::mail::Letter;


use std::env;
use std::sync::Arc;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let letters_rep = MongoRepository::new(&mongo_uri, "mail", "letters").await;

    let mut state = ServiceState::new("mail".to_string());
    state.insert::<Letter>(Arc::new(letters_rep));

    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3007))?
        .run()
        .await
}
