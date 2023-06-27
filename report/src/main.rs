use actix_web::HttpServer;
use report::create_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(create_app)
        .bind(("0.0.0.0", 3011))?
        .run()
        .await
}
