use std::{
    future::Future,
    pin::Pin,
    collections::HashMap,
};
use serde_json::json;
use actix_web::{
    delete, get, patch, post,
    web::{Json, Query, Path},
    HttpResponse,
};

use common::{
    api::{
        audits::{AuditChange, CreateIssue, NoCustomerAuditRequest, PublicAudit},
        seartch::PaginationParams,
    },
    context::GeneralContext,
    entities::{
        audit::{PublicAuditEditHistory, ChangeAuditHistory, EditHistoryResponse},
        issue::ChangeIssue,
        role::Role
    },
    error,
    retry::retry_operation,
};

use crate::service::{
    audit::{AuditService},
    audit_request::PublicRequest,
};

#[post("/audit")]
pub async fn post_audit(
    context: GeneralContext,
    Json(data): Json<PublicRequest>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(AuditService::new(context).create(data).await?))
}

#[post("/no_customer_audit")]
pub async fn post_no_customer_audit(
    context: GeneralContext,
    Json(data): Json<NoCustomerAuditRequest>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(
        AuditService::new(context).create_no_customer(data).await?,
    ))
}

#[get("/audit/{id}")]
pub async fn get_audit(
    context: GeneralContext,
    id: Path<String>,
    query: Query<HashMap<String, String>>,
) -> error::Result<HttpResponse> {
    let code = query.get("code");
    let audit = AuditService::new(context).find(id.parse()?, code).await?;

    if let Some(audit) = audit {
        Ok(HttpResponse::Ok().json(audit))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/my_audit/{role}")]
pub async fn get_my_audit(
    context: GeneralContext,
    role: Path<Role>,
    pagination: Query<PaginationParams>,
) -> error::Result<Json<Vec<PublicAudit>>> {
    Ok(Json(
        AuditService::new(context)
            .my_audit(role.into_inner(), pagination.into_inner())
            .await?,
    ))
}

#[patch("/audit/{id}")]
pub async fn patch_audit(
    context: GeneralContext,
    id: Path<String>,
    Json(data): Json<AuditChange>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(
        AuditService::new(context).change(id.parse()?, data).await?,
    ))
}

#[delete("/audit/{id}")]
pub async fn delete_audit(
    context: GeneralContext,
    id: Path<String>,
) -> error::Result<Json<PublicAudit>> {
    Ok(Json(AuditService::new(context).delete(id.parse()?).await?))
}

#[post("/audit/{id}/issue")]
pub async fn post_audit_issue(
    context: GeneralContext,
    id: Path<String>,
    Json(data): Json<CreateIssue>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context)
        .create_issue(id.parse()?, data)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[patch("/audit/{id}/issue/{issue_id}")]
pub async fn patch_audit_issue(
    context: GeneralContext,
    id: Path<(String, usize)>,
    Json(data): Json<ChangeIssue>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context)
        .change_issue(id.0.parse()?, id.1, data)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/audit/{id}/issue")]
pub async fn get_audit_issue(
    context: GeneralContext,
    id: Path<String>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context).get_issues(id.parse()?).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[get("/audit/{id}/issue/{issue_id}")]
pub async fn get_audit_issue_by_id(
    context: GeneralContext,
    id: Path<(String, usize)>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context)
        .get_issue_by_id(id.0.parse()?, id.1)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[delete("/audit/{id}/issue/{issue_id}")]
pub async fn delete_audit_issue(
    context: GeneralContext,
    id: Path<(String, usize)>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context)
        .delete_issue(id.0.parse()?, id.1)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[patch("/audit/{id}/disclose_all")]
pub async fn patch_audit_disclose_all(
    context: GeneralContext,
    id: Path<String>,
) -> error::Result<HttpResponse> {
    let result = AuditService::new(context).disclose_all(id.parse()?).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[patch("/audit/{id}/{issue_id}/read/{read}")]
pub async fn patch_audit_issue_read(
    context: GeneralContext,
    id: Path<(String, usize, u64)>,
) -> error::Result<HttpResponse> {
    AuditService::new(context)
        .read_events(id.0.parse()?, id.1, id.2)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/public_audits/{user_id}/{role}")]
pub async fn get_public_audits(
    context: GeneralContext,
    path: Path<(String, String)>,
) -> error::Result<Json<Vec<PublicAudit>>> {
    let (user_id, role) = path.into_inner();
    Ok(Json(
        AuditService::new(context)
            .find_public(user_id.parse()?, role)
            .await?,
    ))
}

#[get("/audit/user/{user_id}/{role}")]
pub async fn get_audits_by_user(
    context: GeneralContext,
    path: Path<(String, String)>,
) -> error::Result<Json<Vec<PublicAudit>>> {
    let (user_id, role) = path.into_inner();
    Ok(Json(
        AuditService::new(context)
            .find_audits_by_user(user_id.parse()?, role)
            .await?,
    ))
}

#[get("/audit/{audit_id}/edit_history")]
pub async fn get_audit_edit_history(
    context: GeneralContext,
    audit_id: Path<String>,
) -> error::Result<Json<EditHistoryResponse>> {
    Ok(Json(
        AuditService::new(context)
            .get_audit_edit_history(audit_id.parse()?)
            .await?
    ))
}

#[patch("/audit/{audit_id}/edit_history/{history_id}")]
pub async fn change_audit_edit_history(
    context: GeneralContext,
    params: Path<(String, usize)>,
    Json(data): Json<ChangeAuditHistory>,
) -> error::Result<Json<PublicAuditEditHistory>> {
    // let audit_id = params.0.clone();
    // let history_id = params.1;
    // let result = retry_operation(
    //     move || {
    //         let context_clone = context.clone();
    //         let audit_id = audit_id.clone();
    //         let data = data.clone();
    //         Box::pin(async move {
    //             AuditService::new(context_clone)
    //                 .change_audit_edit_history(audit_id.parse()?, history_id, data)
    //                 .await
    //         }) as Pin<Box<dyn Future<Output = error::Result<PublicAuditEditHistory>> + Send>>
    //     }
    // ).await?;
    //
    // Ok(Json(result))

    Ok(Json(
        AuditService::new(context)
            .change_audit_edit_history(params.0.parse()?, params.1, data)
            .await?
    ))
}

#[patch("/audit/{audit_id}/unread/{unread}")]
pub async fn audit_unread_edits(
    context: GeneralContext,
    params: Path<(String, usize)>,
) -> error::Result<HttpResponse> {
    // let audit_id = params.0.clone();
    // let unread = params.1;
    // retry_operation(
    //     {
    //         move || {
    //             let context_clone = context.clone();
    //             let audit_id = audit_id.clone();
    //             Box::pin(async move {
    //                 AuditService::new(context_clone)
    //                     .unread_edits(audit_id.parse()?, unread)
    //                     .await
    //             }) as Pin<Box<dyn Future<Output = error::Result<()>> + Send>>
    //         }
    //     }
    // ).await?;

    AuditService::new(context)
        .unread_edits(params.0.parse()?, params.1)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
