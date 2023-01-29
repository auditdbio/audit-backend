use std::collections::HashMap;

use actix_web::{
    delete, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use common::{
    auth_session::get_auth_session,
    entities::{audit_request::AuditRequest, role::Role},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Result;
use crate::{handlers::parse_id, repositories::audit_request::AuditRequestRepo};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostAuditRequestRequest {
    pub opener: Role,
    pub auditor_id: String,
    pub customer_id: String,
    pub project_id: String,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub comment: Option<String>,
    pub price: Option<String>,
}

#[utoipa::path(
    request_body(
        content = PostAuditRequestRequest
    ),
    responses(
        (status = 200, body = Auditor)
    )
)]
#[post("/api/requests")]
pub async fn post_audit_request(
    req: HttpRequest,
    Json(data): web::Json<PostAuditRequestRequest>,
    repo: web::Data<AuditRequestRepo>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let audit_request = AuditRequest {
        id: ObjectId::new(),
        auditor_id: parse_id(&data.auditor_id)?,
        customer_id: parse_id(&data.customer_id)?,
        project_id: parse_id(&data.project_id)?,
        auditor_contacts: data.auditor_contacts,
        customer_contacts: data.customer_contacts,
        comment: data.comment,
        last_modified: Utc::now().naive_utc(),
        price: data.price,
    };

    repo.create(&audit_request).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetAuditRequestsResponse {
    pub audits: Vec<AuditRequest>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PatchAuditRequestRequest {
    pub id: ObjectId,
    pub auditor_contacts: Option<HashMap<String, String>>,
    pub customer_contacts: Option<HashMap<String, String>>,
    pub comment: Option<String>,
    pub price: Option<String>,
}

#[utoipa::path(
    request_body(
        content = PatchAuditRequestRequest
    ),
    responses(
        (status = 200, body = Auditor)
    )
)]
#[patch("/api/requests")]
pub async fn patch_audit_request(
    req: HttpRequest,
    Json(data): web::Json<PatchAuditRequestRequest>,
    repo: web::Data<AuditRequestRepo>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(mut audit_request) = repo.delete(&data.id).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if let Some(auditor_contacts) = data.auditor_contacts {
        audit_request.auditor_contacts = auditor_contacts;
    }

    if let Some(customer_contacts) = data.customer_contacts {
        audit_request.customer_contacts = customer_contacts;
    }

    if let Some(comment) = data.comment {
        audit_request.comment = Some(comment);
    }

    if let Some(price) = data.price {
        audit_request.price = Some(price);
    }

    audit_request.last_modified = Utc::now().naive_utc();

    repo.create(&audit_request).await?;

    Ok(HttpResponse::Ok().json(audit_request))
}

#[utoipa::path(
    responses(
        (status = 200, body = Auditor)
    )
)]
#[delete("/api/requests/{id}")]
pub async fn delete_audit_request(
    req: HttpRequest,
    id: web::Path<ObjectId>,
    repo: web::Data<AuditRequestRepo>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(request) = repo.delete(&id).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    Ok(HttpResponse::Ok().json(request))
}
