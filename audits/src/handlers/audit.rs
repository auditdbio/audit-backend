use actix_web::{
    delete, get, patch, post,
    web::{self, Json, Query},
    HttpResponse,
};

use common::{
    api::{
        audits::{AuditChange, CreateIssue, PublicAudit, NoCustomerAuditRequest},
        seartch::PaginationParams,
    },
    context::GeneralContext,
    entities::{issue::ChangeIssue, role::Role},
    error,
};

use serde_json::json;

use crate::service::{
    audit::{AuditService, MyAuditResult},
    audit_request::PublicRequest
};

#[post("/api/audit")]
pub async fn post_audit(
    context: GeneralContext,
    Json(data): web::Json<PublicRequest>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(AuditService::new(context).create(data).await?))
}

#[post("/api/no_customer_audit")]
pub async fn post_no_customer_audit(
    context: GeneralContext,
    Json(data): Json<NoCustomerAuditRequest>
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(AuditService::new(context).create_no_customer(data).await?))
}

#[get("/api/audit/{id}")]
pub async fn get_audit(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let res = AuditService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/my_audit/{role}")]
pub async fn get_my_audit(
    context: GeneralContext,
    role: web::Path<Role>,
    pagination: Query<PaginationParams>,
) -> error::Result<Json<MyAuditResult>> {
    Ok(Json(
        AuditService::new(context)
            .my_audit(role.into_inner(), pagination.into_inner())
            .await?,
    ))
}

#[patch("/api/audit/{id}")]
pub async fn patch_audit(
    context: GeneralContext,
    id: web::Path<String>,
    Json(data): Json<AuditChange>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(
        AuditService::new(context).change(id.parse()?, data).await?,
    ))
}

#[delete("/api/audit/{id}")]
pub async fn delete_audit(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(AuditService::new(context).delete(id.parse()?).await?))
}

#[post("/api/audit/{id}/issue")]
pub async fn post_audit_issue(
    context: GeneralContext,
    id: web::Path<String>,
    Json(data): Json<CreateIssue>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context)
        .create_issue(id.parse()?, data)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[patch("/api/audit/{id}/issue/{issue_id}")]
pub async fn patch_audit_issue(
    context: GeneralContext,
    id: web::Path<(String, usize)>,
    Json(data): Json<ChangeIssue>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context)
        .change_issue(id.0.parse()?, id.1, data)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/api/audit/{id}/issue")]
pub async fn get_audit_issue(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context).get_issues(id.parse()?).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/api/audit/{id}/issue/{issue_id}")]
pub async fn get_audit_issue_by_id(
    context: GeneralContext,
    id: web::Path<(String, usize)>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context)
        .get_issue_by_id(id.0.parse()?, id.1)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[delete("/api/audit/{id}/issue/{issue_id}")]
pub async fn delete_audit_issue(
    context: GeneralContext,
    id: web::Path<(String, usize)>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context).delete_issue(id.0.parse()?, id.1).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[patch("/api/audit/{id}/disclose_all")]
pub async fn patch_audit_disclose_all(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context).disclose_all(id.parse()?).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[patch("/api/audit/{id}/{issue_id}/read/{read}")]
pub async fn patch_audit_issue_read(
    context: GeneralContext,
    id: web::Path<(String, usize, u64)>,
) -> error::Result<HttpResponse> {
    AuditService::new(context)
        .read_events(id.0.parse()?, id.1, id.2)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/api/public_audits/{id}/{role}")]
pub async fn get_public_audits(
    context: GeneralContext,
    path: web::Path<(String, String)>,
) -> error::Result<Json<Vec<PublicAudit>>> {
    let (id, role) = path.into_inner();
    Ok(Json(
        AuditService::new(context)
            .find_public(id.parse()?, role)
            .await?,
    ))
}
