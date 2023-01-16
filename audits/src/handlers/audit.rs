use actix_web::{HttpRequest, HttpResponse, delete, get, web::{self, Json}};
use common::{auth_session::get_auth_session};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::{error::Result, repositories::audit::AuditRepo};

use super::parse_id;

#[get("/api/audit")]
pub async fn get_audits(req: HttpRequest, repo: web::Data<AuditRepo>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let audits = repo.get_audits(&session.user_id()).await?;

    Ok(HttpResponse::Ok().json(audits))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchAuditRequest {
    pub id: String,
    pub customer_id: Option<String>,
    pub auditor_id: Option<String>,
    pub project_id: Option<String>,
    pub terms: Option<String>,
    pub status: Option<String>,
}

pub async fn patch_audit(
    req: HttpRequest,
    Json(data): Json<PatchAuditRequest>,
    repo: web::Data<AuditRepo>,
) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let mut audit = repo.delete(parse_id(&data.id)?).await?.unwrap();

    if let Some(customer_id) = data.customer_id {
        let old_customer = audit.customer_id;
        audit.customer_id = parse_id(&customer_id)?;
        audit.visibility.iter_mut().for_each(|v| {
            if v == &old_customer {
                *v = audit.customer_id;
            }
        });
    }
    
    if let Some(auditor_id) = data.auditor_id {
        let old_auditor = audit.auditor_id;
        audit.auditor_id = parse_id(&auditor_id)?;
        audit.visibility.iter_mut().for_each(|v| {
            if v == &old_auditor {
                *v = audit.auditor_id;
            }
        });
    }

    if let Some(project_id) = data.project_id {
        audit.project_id = parse_id(&project_id)?;
    }

    if let Some(terms) = data.terms {
        audit.terms = terms;
    }

    if let Some(status) = data.status {
        audit.status = status;
    }

    audit.last_modified = chrono::Utc::now().naive_utc();

    repo.create(&audit).await?;

    Ok(HttpResponse::Ok().json(audit))
}

#[delete("/api/auditors")]
pub async fn delete_audit(req: HttpRequest, repo: web::Data<AuditRepo>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.delete(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };


    Ok(HttpResponse::Ok().json(auditor))
}
