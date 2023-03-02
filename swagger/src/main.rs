use std::{error::Error, net::Ipv4Addr};

use actix_web::{middleware::Logger, App, HttpServer};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::{SwaggerUi, Url};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "http",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();

    #[derive(OpenApi)]
    #[openapi(
        servers(
            (url = "http://dev.auditdb.io/"),
        ),
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
            users::GetUsersResponse,
            common::entities::user::User,
            common::auth_session::AuthSession
        ))
    )]
    struct UsersServiceDoc;

    #[derive(OpenApi)]
    #[openapi(
        servers(
            (url = "http://dev.auditdb.io/"),
        ),
        paths(
            customers::post_customer,
            customers::get_customer,
            customers::patch_customer,
            customers::delete_customer,
            customers::post_project,
            customers::patch_project,
            customers::delete_project,
            customers::get_project,
            customers::get_projects,
        ),
        components(schemas(
            customers::PostCustomerRequest,
            customers::PatchCustomerRequest,
            customers::PostProjectRequest,
            customers::PatchProjectRequest,
            customers::AllProjectsResponse,
            common::entities::customer::Customer,
        ))
    )]
    struct CustomersServiceDoc;

    #[derive(OpenApi)]
    #[openapi(
        servers(
            (url = "http://dev.auditdb.io/"),
        ),
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
            auditors::AllAuditorsResponse,
            common::entities::auditor::Auditor,
        ))
    )]
    struct AuditorsServiceDoc;

    #[derive(OpenApi)]
    #[openapi(
        servers(
            (url = "http://dev.auditdb.io/"),
        ),
        paths(
            audits::post_audit,
            audits::get_audits,
            audits::delete_audit,
            audits::get_views,
            audits::post_audit_request,
            audits::patch_audit_request,
            audits::delete_audit_request,
        ),
        components(schemas(
            audits::PostAuditRequestRequest,
            audits::PatchAuditRequestRequest,
            audits::GetAuditRequestsResponse,
            audits::GetViewsResponse,
            common::entities::audit::Audit,
            common::entities::audit_request::AuditRequest,
            common::entities::view::View,
        ))
    )]
    struct AuditsServiceDoc;

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
                (
                    Url::new("audits", "/api-doc/openapi4.json"),
                    AuditsServiceDoc::openapi(),
                ),
            ]))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
