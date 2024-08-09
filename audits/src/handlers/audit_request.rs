use actix_web::{
    delete, get, patch, post,
    web::{self, Json, Query},
    HttpResponse,
};

use common::{
    api::{
        requests::{CreateRequest, PublicRequest},
        seartch::PaginationParams,
    },
    context::GeneralContext,
    entities::{
        audit::EditHistoryResponse,
        audit_request::AuditRequest,
        role::Role
    },
    error,
};

use serde_json::json;

use crate::service::audit_request::{RequestChange, RequestService};

#[post("/audit_request")]
pub async fn post_audit_request(
    context: GeneralContext,
    Json(data): web::Json<CreateRequest>,
) -> error::Result<Json<PublicRequest>> {
    Ok(Json(RequestService::new(context).create(data).await?))
}

#[get("/audit_request/{id}")]
pub async fn get_audit_request(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let res = RequestService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/my_audit_request/{role}")]
pub async fn get_my_audit_request(
    context: GeneralContext,
    role: web::Path<Role>,
    pagination: Query<PaginationParams>,
) -> error::Result<Json<Vec<PublicRequest>>> {
    Ok(Json(
        RequestService::new(context)
            .my_request(role.into_inner(), pagination.into_inner())
            .await?,
    ))
}

#[patch("/audit_request/{id}")]
pub async fn patch_audit_request(
    context: GeneralContext,
    id: web::Path<String>,
    Json(data): Json<RequestChange>,
) -> error::Result<Json<PublicRequest>> {
    Ok(Json(
        RequestService::new(context)
            .change(id.parse()?, data)
            .await?,
    ))
}

#[delete("/audit_request/{id}")]
pub async fn delete_audit_request(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<Json<PublicRequest>> {
    Ok(Json(
        RequestService::new(context).delete(id.parse()?).await?,
    ))
}

#[get("/audit_request/all/{role}/{id}")]
pub async fn find_all_audit_request(
    context: GeneralContext,
    path: web::Path<(Role, String)>,
) -> error::Result<Json<Vec<AuditRequest<String>>>> {
    let (role, id) = path.into_inner();
    Ok(Json(
        RequestService::new(context)
            .find_all(role, id.parse()?)
            .await?,
    ))
}

#[get("/audit_request/organization/all")]
pub async fn get_my_organization_audit_requests(
    context: GeneralContext,
) -> error::Result<Json<Vec<PublicRequest>>> {
    Ok(Json(
        RequestService::new(context)
            .find_my_organization_audit_requests()
            .await?,
    ))
}

#[get("/audit_request/organization/{org_id}")]
pub async fn get_organization_audit_requests(
    context: GeneralContext,
    path: web::Path<String>,
) -> error::Result<Json<Vec<PublicRequest>>> {
    let org_id = path.into_inner();
    Ok(Json(
        RequestService::new(context)
            .find_organization_audit_requests(org_id.parse()?)
            .await?,
    ))
}

#[get("/audit_request/{id}/edit_history")]
pub async fn get_request_edit_history(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<Json<EditHistoryResponse>> {
    Ok(Json(
        RequestService::new(context)
            .get_request_edit_history(id.parse()?)
            .await?
    ))
}

#[patch("/audit_request/{request_id}/unread/{unread}")]
pub async fn request_unread_edits(
    context: GeneralContext,
    params: web::Path<(String, usize)>,
) -> error::Result<HttpResponse> {
    RequestService::new(context)
        .unread_edits(params.0.parse()?, params.1)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
