use std::collections::HashMap;

use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use common::{
    auth_session::{AuthSessionManager, SessionManager},
    entities::{
        audit_request::{AuditRequest, PriceRange, TimeRange},
        role::Role,
    },
};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{repositories::audit_request::AuditRequestRepo, handlers::audit::get_auditor};
use crate::{error::Result, handlers::audit::get_project};
use crate::{handlers::audit::get_auditor, repositories::audit_request::AuditRequestRepo};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostAuditRequestRequest {
    pub auditor_id: String,
    pub customer_id: String,
    pub project_id: String,
    pub description: String,
    pub auditor_contacts: HashMap<String, String>,
    pub customer_contacts: HashMap<String, String>,
    pub scope: Vec<String>,
    pub price: Option<String>,
    pub price_range: Option<PriceRange>,
    pub time_frame: String,
    pub time: TimeRange,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PostAuditRequestRequest
    ),
    responses(
        (status = 200, body = AuditRequest<String>)
    )
)]
#[post("/api/requests")]
pub async fn post_audit_request(
    req: HttpRequest,
    Json(data): web::Json<PostAuditRequestRequest>,
    repo: web::Data<AuditRequestRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.clone().into()).await.unwrap(); // TODO: remove unwrap

    let auditor_id = data.auditor_id.parse().unwrap();

    let customer_id = data.customer_id.parse().unwrap();

    let project_id = data.project_id.parse().unwrap();

    let mut client = awc::Client::default();

    client.headers().unwrap().insert(
        "Authorization".parse().unwrap(),
        req.headers().get("Authorization").unwrap().clone(),
    );
    let Some(result) = get_project(&client, &project_id).await else {
        return Ok(HttpResponse::Ok().json(doc!{"error": "project id is invalid"}));
    };
    let project = result.unwrap();

    let Some(result) = get_auditor(&client, &auditor_id).await else {
        return Ok(HttpResponse::Ok().json(doc!{"error": "project id is invalid"}));
    };
    let auditor = result.unwrap();

    let last_changer = if &session.user_id == &auditor_id {
        Role::Auditor
    } else if &session.user_id == &customer_id {
        Role::Customer
    } else {
        return Ok(
            HttpResponse::Ok().json(doc! {"Error": "You are not allowed to change this request"})
        );
    };

    let Some(result) = get_auditor(&client, &auditor_id).await else {
        return Ok(HttpResponse::Ok().json(doc!{"error": "auditor id is invalid"}));
    };
    let auditor = result.unwrap();

    let mut audit_request = AuditRequest {
        id: ObjectId::new(),
        customer_id: customer_id,
        auditor_id: auditor_id,
        project_id: project_id,
        project_name: project.name,
        avatar: auditor.avatar,
        description: Some(data.description),
        auditor_contacts: data.auditor_contacts,
        customer_contacts: data.customer_contacts,
        scope: data.scope,
        price: data.price,
        time_frame: data.time_frame,
        last_modified: Utc::now().naive_utc().timestamp_micros(),
        last_changer: last_changer,
        time: data.time,
    };

    let old_request = repo
        .find_by_auditor(audit_request.auditor_id)
        .await?
        .into_iter()
        .filter(|request| {
            &request.customer_id == &audit_request.customer_id
                && &request.project_id == &audit_request.project_id
        })
        .next();

    if let Some(old_request) = old_request {
        repo.delete(&old_request.id).await?;
        audit_request.id = old_request.id;
    }

    repo.create(&audit_request).await?;

    Ok(HttpResponse::Ok().json(audit_request.stringify()))
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetAuditRequestsQuery {
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetAuditRequestsResponse {
    pub audits: Vec<AuditRequest<String>>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
        GetAuditRequestsQuery
    ),
    responses(
        (status = 200, body = AuditRequest<String>)
    )
)]
#[get("/api/requests")]
pub async fn get_audit_requests(
    req: HttpRequest,
    query: web::Query<GetAuditRequestsQuery>,
    repo: web::Data<AuditRequestRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let audits = match query.role.as_str() {
        "Auditor" | "auditor" => repo.find_by_auditor(session.user_id).await?,
        "Customer" | "customer" => repo.find_by_customer(session.user_id).await?,
        _ => {
            unreachable!()
        }
    };

    Ok(HttpResponse::Ok().json(GetAuditRequestsResponse {
        audits: audits.into_iter().map(AuditRequest::stringify).collect(),
    }))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PatchAuditRequestRequest {
    pub id: String,
    pub auditor_contacts: Option<HashMap<String, String>>,
    pub customer_contacts: Option<HashMap<String, String>>,
    pub scope: Option<Vec<String>>,
    pub price: Option<String>,
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
        (status = 200, body = AuditRequest<String>)
    )
)]
#[patch("/api/requests")]
pub async fn patch_audit_request(
    req: HttpRequest,
    Json(data): web::Json<PatchAuditRequestRequest>,
    repo: web::Data<AuditRequestRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap
    let id = data.id.parse().unwrap();

    let Some(mut audit_request) = repo.delete(&id).await? else {
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

    if let Some(time_frame) = data.time_frame {
        audit_request.time_frame = time_frame;
    }

    let last_changer = if &session.user_id == &audit_request.auditor_id {
        Role::Auditor
    } else if &session.user_id == &audit_request.customer_id {
        Role::Customer
    } else {
        return Ok(
            HttpResponse::Ok().json(doc! {"Error": "You are not allowed to change this request"})
        );
    };

    audit_request.last_changer = last_changer;

    audit_request.last_modified = Utc::now().naive_utc().timestamp_micros();

    repo.create(&audit_request).await?;

    Ok(HttpResponse::Ok().json(audit_request.stringify()))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = AuditRequest<String>)
    )
)]
#[delete("/api/requests/{id}")]
pub async fn delete_audit_request(
    req: HttpRequest,
    id: web::Path<String>,
    repo: web::Data<AuditRequestRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let _session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(request) = repo.delete(&id.parse::<ObjectId>().unwrap()).await? else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };
    Ok(HttpResponse::Ok().json(request.stringify()))
}

#[utoipa::path(
    responses(
        (status = 200, body = Project<String>)
    )
)]
#[get("/api/requests/by_id/{id}")]
pub async fn requests_by_id(
    id: web::Path<String>,
    repo: web::Data<AuditRequestRepo>,
) -> Result<HttpResponse> {
    let Some(request) = repo.find(id.parse().unwrap()).await? else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };
    Ok(HttpResponse::Ok().json(request.stringify()))
}
