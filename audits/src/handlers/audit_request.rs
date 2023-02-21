use std::collections::HashMap;

use actix_web::{
    delete, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use common::{
    auth_session::get_auth_session,
    entities::{
        audit_request::{AuditRequest, PriceRange},
        role::Role,
    },
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Result;
use crate::repositories::audit_request::AuditRequestRepo;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostAuditRequestRequest {
    pub auditor_id: ObjectId,
    pub customer_id: ObjectId,
    pub project_id: ObjectId,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub scope: Vec<String>,
    pub price: Option<i64>,
    pub price_range: Option<PriceRange>,
    pub time_frame: String,
    pub opener: Role,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PostAuditRequestRequest
    ),
    responses(
        (status = 200, body = Audit)
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
        customer_id: data.customer_id,
        auditor_id: data.auditor_id,
        project_id: data.project_id,
        auditor_contacts: data.auditor_contacts,
        customer_contacts: data.customer_contacts,
        scope: data.scope,
        price: data.price,
        price_range: data.price_range,
        time_frame: data.time_frame,
        last_modified: Utc::now().naive_utc(),
        opener: data.opener,
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
    pub scope: Option<Vec<String>>,
    pub price: Option<i64>,
    pub price_range: Option<PriceRange>,
    pub time_frame: Option<String>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
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

    if let Some(scope) = data.scope {
        audit_request.scope = scope;
    }

    if let Some(price) = data.price {
        audit_request.price = Some(price);
    }

    if let Some(price_range) = data.price_range {
        audit_request.price_range = Some(price_range);
    }

    if let Some(time_frame) = data.time_frame {
        audit_request.time_frame = time_frame;
    }

    audit_request.last_modified = Utc::now().naive_utc();

    repo.create(&audit_request).await?;

    Ok(HttpResponse::Ok().json(audit_request))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
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
