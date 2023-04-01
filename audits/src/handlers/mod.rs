use crate::{
    error::Result,
    repositories::{audit::AuditRepo, audit_request::AuditRequestRepo},
};
use actix_web::{
    get,
    web::{self},
    HttpRequest, HttpResponse,
};
use common::{
    auth_session::{AuthSessionManager, Role, SessionManager},
    entities::{audit::Audit, audit_request::AuditRequest},
};

pub mod audit;
pub mod audit_request;

#[get("/api/audit/data/{resource}/{timestamp}")]
pub async fn get_data(
    req: HttpRequest,
    since: web::Path<(String, i64)>,
    audit_repo: web::Data<AuditRepo>,
    request_repo: web::Data<AuditRequestRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let (resource, since) = since.into_inner();
    // let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap
    // if session.role != Role::Service {
    //     return Ok(HttpResponse::Unauthorized().finish());
    // }

    match resource.as_str() {
        "audit" => {
            let audits = audit_repo.get_all_since(since).await?;
            Ok(HttpResponse::Ok().json(
                audits
                    .into_iter()
                    .map(Audit::stringify)
                    .map(Audit::to_doc)
                    .collect::<Vec<_>>(),
            ))
        }
        "request" => {
            let requests = request_repo.get_all_since(since).await?;
            Ok(HttpResponse::Ok().json(
                requests
                    .into_iter()
                    .map(AuditRequest::stringify)
                    .map(AuditRequest::to_doc)
                    .collect::<Vec<_>>(),
            ))
        }
        _ => Ok(HttpResponse::NotFound().finish()),
    }
}
