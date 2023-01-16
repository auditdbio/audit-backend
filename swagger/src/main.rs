use std::{error::Error, net::Ipv4Addr};

use actix_web::{middleware::Logger, web, App, HttpServer};
use customers::PostCustomerRequest;
use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme}, ToSchema};
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
        components(schemas(
            users::LoginRequest,
            users::LoginResponse,
            users::RestoreResponse,
            users::PostUserRequest,
            users::PatchUserRequest,
            users::GetUsersRequest,
            common::entities::user::User,
            common::auth_session::AuthSession
        ))
    )]
    struct UsersServiceDoc;

    #[derive(OpenApi)]
    #[openapi(
        paths(
            customers::post_customer,
            customers::get_customer,
            customers::patch_customer,
            customers::delete_customer,
            customers::post_project,
            customers::patch_project,
            customers::delete_project,
            customers::get_project,
        ),
        components(schemas(
            customers::PostCustomerRequest,
            customers::PatchCustomerRequest,
            customers::PostProjectRequest,
            customers::PatchProjectRequest,
            common::entities::customer::Customer
        ))
    )]
    struct CustomersServiceDoc;


    #[derive(OpenApi)]
    #[openapi(
        paths(
            auditors::post_auditor,
            auditors::get_auditor,
            auditors::patch_auditor,
            auditors::delete_auditor,
            auditors::get_auditors,
        ),
        components(schemas(
            auditors::PostAuditorRequest,
            auditors::PatchAuditorRequest,
            auditors::AllAuditorsRequest,
            common::entities::auditor::Auditor,
        ))
    )]
    struct AuditorsServiceDoc;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
                (
                    Url::new("users", "/api-doc/openapi1.json"),
                    UsersServiceDoc::openapi(),
                ),
                (
                    Url::new("customers", "/api-doc/openapi2.json"),
                    CustomersServiceDoc::openapi(),
                ),
                (
                    Url::new("auditors", "/api-doc/openapi3.json"),
                    AuditorsServiceDoc::openapi(),
                ),
            ]))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
