use std::{env, sync::Arc};

use actix_web::HttpServer;
use chat::{create_app, repositories::chat::ChatRepository};
use common::{context::ServiceState, repository::mongo_repository::MongoRepository};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let state = ServiceState::new("chat".to_string());

    let mongo_uri = env::var("MONGOURI").unwrap();

    let messages = MongoRepository::new(&mongo_uri, "chat", "messages").await;
    let groups = MongoRepository::new(&mongo_uri, "chat", "groups").await;
    let private_chats = MongoRepository::new(&mongo_uri, "chat", "private_chats").await;
    let chat = Arc::new(ChatRepository::new(messages, groups, private_chats));

    let mut state = state;

    state.insert_manual::<Arc<ChatRepository>>(chat);

    let state = Arc::new(state);

    HttpServer::new(move || create_app(state.clone()))
        .bind(("0.0.0.0", 3012))?
        .run()
        .await
}
