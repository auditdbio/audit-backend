use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use common::{
    context::Context,
    entities::{issue::ChangeIssue, role::Role},
    error,
};

use serde_json::json;

use crate::service::{
    audit::{AuditChange, AuditService, CreateIssue, PublicAudit},
    audit_request::PublicRequest,
};

#[post("/api/audit")]
pub async fn post_audit(
    context: Context,
    Json(data): web::Json<PublicRequest>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(AuditService::new(context).create(data).await?))
}

#[get("/api/audit/{id}")]
pub async fn get_audit(context: Context, id: web::Path<String>) -> error::Result<HttpResponse> {
    let res = AuditService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/my_audit/{role}")]
pub async fn get_my_audit(
    context: Context,
    role: web::Path<Role>,
) -> error::Result<Json<Vec<PublicAudit>>> {
    Ok(Json(
        AuditService::new(context)
            .my_audit(role.into_inner())
            .await?,
    ))
}

#[patch("/api/audit/{id}")]
pub async fn patch_audit(
    context: Context,
    id: web::Path<String>,
    Json(data): Json<AuditChange>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(
        AuditService::new(context).change(id.parse()?, data).await?,
    ))
}

#[delete("/api/audit/{id}")]
pub async fn delete_audit(
    context: Context,
    id: web::Path<String>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(AuditService::new(context).delete(id.parse()?).await?))
}

#[post("/api/audit/{id}/issue")]
pub async fn post_audit_issue(
    context: Context,
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
    context: Context,
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
    context: Context,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context).get_issues(id.parse()?).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/api/audit/{id}/issue/{issue_id}")]
pub async fn get_audit_issue_by_id(
    context: Context,
    id: web::Path<(String, usize)>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context)
        .get_issue_by_id(id.0.parse()?, id.1)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}
