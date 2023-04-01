use actix_web::{
    get,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{
    auth_session::{AuthSessionManager, Role, SessionManager},
    entities::auditor::Auditor,
};
use mongodb::bson::Document;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::repositories::auditor::AuditorRepo;

pub mod auditor;

#[get("/api/auditor/data/{resource}/{timestamp}")]
pub async fn get_data(
    req: HttpRequest,
    args: web::Path<(String, i64)>,
    repo: web::Data<AuditorRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let (resource, since) = args.into_inner();
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap
    if session.role != Role::Service {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    return match resource.as_str() {
        "auditor" => {
            let auditors = repo.get_all_since(since).await?;

            Ok(HttpResponse::Ok().json(
                auditors
                    .into_iter()
                    .map(Auditor::stringify)
                    .map(Auditor::to_doc)
                    .collect::<Vec<_>>(),
            ))
        }
        _ => Ok(HttpResponse::NotFound().finish()),
    };
}
