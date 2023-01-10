use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, post, patch, delete, get, web::{self, Json}};
use common::auth_session::get_auth_session;
use serde::{Serialize, Deserialize};

use crate::{repositories::auditor::{AuditorRepository, AuditorModel}, error::Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostAuditorRequest {
    first_name: String,
    last_name: String,
    about: String,
    tags: Vec<String>,
    contacts: HashMap<String, String>,
}

#[post("/api/auditors")]
pub async fn post_auditor(req: HttpRequest, Json(data): web::Json<PostAuditorRequest>, repo: web::Data<AuditorRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let auditor = AuditorModel {
        user_id: session.user_id(),
        first_name: data.first_name,
        last_name: data.last_name,
        about: data.about,
        contacts: data.contacts,
        tags: data.tags,
    };

    if !repo.create(auditor).await? {
        return Ok(HttpResponse::BadRequest().finish());
    }
    Ok(HttpResponse::Ok().finish())
}

#[get("/api/auditors")]
pub async fn get_auditor(req: HttpRequest, repo: web::Data<AuditorRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    todo!()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchAuditorRequest {
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    tags: Option<Vec<String>>,
    contacts: Option<HashMap<String, String>>,
}

#[patch("/api/auditors")]
pub async fn patch_auditor(req: HttpRequest, repo: web::Data<AuditorRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.find(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    Ok(HttpResponse::Ok().json(auditor))
}

#[delete("/api/auditors")]
pub async fn delete_auditor(req: HttpRequest, repo: web::Data<AuditorRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.delete(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    Ok(HttpResponse::Ok().json(auditor))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllAuditorsRequest {
    tags: Vec<String>,
}

#[get("/api/auditors/all")]
pub async fn get_auditors(
    Json(data): web::Json<AllAuditorsRequest>,
    repo: web::Data<AuditorRepository>,
) -> Result<HttpResponse> {
    let auditors = repo.request_with_tags(data.tags).await?;
    Ok(HttpResponse::Ok().json(auditors))
}
