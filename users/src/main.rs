extern crate lazy_static;

use common::context::ServiceState;
use common::entities::user::User;
use common::repository::mongo_repository::MongoRepository;
use mongodb::bson::oid::ObjectId;
use users::service::auth::{Code, Link};
use users::service::waiting_list::WaitingListElement;
use users::*;

use std::env;
use std::sync::Arc;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let mongo_uri = env::var("MONGOURI").unwrap();

    let user_repo = MongoRepository::new(&mongo_uri, "users", "users").await;
    let link_repo = MongoRepository::new(&mongo_uri, "users", "links").await;
    let waiting_list_repo = MongoRepository::new(&mongo_uri, "users", "waiting_list").await;
    let code_repo = MongoRepository::new(&mongo_uri, "users", "codes").await;

    let mut state = ServiceState::new("user".to_string());
    state.insert::<User<ObjectId>>(Arc::new(user_repo));
    state.insert::<Link>(Arc::new(link_repo));
    state.insert::<WaitingListElement>(Arc::new(waiting_list_repo));
    state.insert::<Code>(Arc::new(code_repo));

    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3001))?
        .run()
        .await
}
