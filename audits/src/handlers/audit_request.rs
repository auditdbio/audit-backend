use std::{collections::HashMap};

use actix_web::{HttpRequest, HttpResponse, post, patch, delete, get, web::{self, Json}};
use common::{auth_session::get_auth_session, entities::{role::Role, audit_request::AuditRequest}};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::{repositories::audit_request::{AuditRequestRepo}, handlers::parse_id};
use crate::error::{Result};

#[derive(Debug, Serialize, Deserialize)]
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

#[post("/api/audits/requests")]
pub async fn post_audit_request(req: HttpRequest, Json(data): web::Json<PostAuditRequestRequest>, repo: web::Data<AuditRequestRepo>) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let audit_request = AuditRequest {
        id: ObjectId::new(),
        opener: data.opener,
        auditor_id: parse_id(&data.auditor_id)?,
        customer_id: parse_id(&data.customer_id)?,
        project_id: parse_id(&data.project_id)?,
        auditor_contacts: data.auditor_contacts,
        customer_contacts: data.customer_contacts,
        comment: data.comment,
        price: data.price,
    };

    repo.create(&audit_request).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAuditRequestsRequest {
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAuditRequestsResponse {
    pub audits: Vec<AuditRequest>,
}

#[get("/api/audits/requests")]
pub async fn get_audit_request(req: HttpRequest, web::Json(data): web::Json<GetAuditRequestsRequest>, repo: web::Data<AuditRequestRepo>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let results = match data.role {
        Role::Auditor => repo.find_by_auditor(session.user_id()).await?,
        Role::Customer => repo.find_by_customer(session.user_id()).await?,
    };

    Ok(HttpResponse::Ok().json(GetAuditRequestsResponse { audits: results }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchAuditorRequest {
    id: ObjectId,
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    tags: Option<Vec<String>>,
    contacts: Option<HashMap<String, String>>,
}

#[patch("/api/audits/requests")]
pub async fn patch_auditor(req: HttpRequest, repo: web::Data<AuditRequestRepo>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.find(&session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    Ok(HttpResponse::Ok().json(auditor))
}

#[delete("/api/audits/requests")]
pub async fn delete_auditor(req: HttpRequest, repo: web::Data<AuditRequestRepo>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.delete(&session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    Ok(HttpResponse::Ok().json(auditor))
}
