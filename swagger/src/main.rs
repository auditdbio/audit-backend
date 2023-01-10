use std::{error::Error, net::Ipv4Addr};

use actix_web::{middleware::Logger, web, App, HttpServer};
use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme}};
use utoipa_swagger_ui::{SwaggerUi, Url};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "http",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build()),
            )
        }
    }
}

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            users::login,
            users::restore,
            users::verify,
            users::post_user,
            users::patch_user,
            users::delete_user,
            users::get_user,
            users::get_users,
        ),
        modifiers(&SecurityAddon),
        components(schemas(
            users::LoginRequest,
            common::AuthSession
        ))
    )]
    struct UsersServiceDoc;


    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
                (
                    Url::new("users", "/api-doc/openapi1.json"),
                    UsersServiceDoc::openapi(),
                ),
            ]))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
