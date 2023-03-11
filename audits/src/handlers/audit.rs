use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use awc::Client;
use chrono::Utc;
use common::{
    auth_session::{AuthSessionManager, SessionManager},
    entities::{audit::Audit, audit_request::AuditRequest, project::Project, view::View},
};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    contants::CUSTOMERS_SERVICE,
    error::Result,
    repositories::{
        audit::AuditRepo, audit_request::AuditRequestRepo, closed_audits::ClosedAuditRepo,
    },
};

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = AuditRequest<String>
    ),
    responses(
        (status = 200, body = Audit<String>)
    )
)]
#[post("/api/audit")]
pub async fn post_audit(
    req: HttpRequest,
    web::Json(request): web::Json<AuditRequest<String>>,
    repo: web::Data<AuditRepo>,
    request_repo: web::Data<AuditRequestRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let _session = manager.get_session(req.clone().into()).await.unwrap(); // TODO: remove unwrap
    let request = request.parse();

    let Some(price) = request.price else {
        return Ok(HttpResponse::BadRequest().body("Price is required"));
    };

    let mut client = awc::Client::default();

    client.headers().unwrap().insert(
        "Authorization".parse().unwrap(),
        req.headers().get("Authorization").unwrap().clone(),
    );

    let project_id: ObjectId = request.project_id;

    let project = get_project(&client, &project_id).await?;

    let audit = Audit {
        id: ObjectId::new(),
        customer_id: request.customer_id,
        auditor_id: request.auditor_id,
        project_id: project_id,
        project_name: project.name,
        description: request.description,
        status: "pending".to_string(),
        last_modified: Utc::now().naive_utc().timestamp_micros(),
        auditor_contacts: request.auditor_contacts,
        customer_contacts: request.customer_contacts,
        price: price,
        report_link: None,
        time_frame: request.time_frame,
        scope: request.scope,
        tags: project.tags,
        report: None,
        time: request.time,
    };

    repo.create(&audit).await?;
    request_repo.delete(&request.id).await?;
    Ok(HttpResponse::Ok().json(audit.stringify()))
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetAuditQuery {
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetAuditResponse {
    pub audits: Vec<Audit<String>>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
        GetAuditQuery,
    ),
    responses(
        (status = 200, body = GetAuditResponse)
    )
)]
#[get("/api/audit")]
pub async fn get_audit(
    req: HttpRequest,
    query: web::Query<GetAuditQuery>,
    repo: web::Data<AuditRepo>,
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

    Ok(HttpResponse::Ok().json(GetAuditResponse {
        audits: audits.into_iter().map(|a| a.stringify()).collect(),
    }))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Audit<String>)
    )
)]
#[delete("/api/audit/{id}")]
pub async fn delete_audit(
    req: HttpRequest,
    id: web::Path<String>,
    repo: web::Data<AuditRepo>,
    closed_repo: web::Data<ClosedAuditRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let _session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(audit) = repo.delete(&id.parse::<ObjectId>().unwrap()).await? else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };

    closed_repo.create(&audit).await?;

    Ok(HttpResponse::Ok().json(audit.stringify()))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetViewsResponse {
    pub views: Vec<View<String>>,
}

pub(super) async fn get_project(client: &Client, project_id: &ObjectId) -> Result<Project<String>> {
    let mut res = client
        .get(format!(
            "https://{}/api/projects/by_id/{}",
            CUSTOMERS_SERVICE,
            project_id.to_hex()
        ))
        .send()
        .await
        .unwrap();
    let body = res.json::<Project<String>>().await.unwrap();
    Ok(body)
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = GetViewsResponse)
    )
)]
#[get("/api/audit/views")]
pub async fn get_views(
    req: HttpRequest,
    request_repo: web::Data<AuditRequestRepo>,
    audits_repo: web::Data<AuditRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.clone().into()).await.unwrap(); // TODO: remove unwrap

    let mut views = Vec::new();
    let mut client = awc::Client::default();

    client.headers().unwrap().insert(
        "Authorization".parse().unwrap(),
        req.headers().get("Authorization").unwrap().clone(),
    );

    let requests_as_auditor = request_repo.find_by_auditor(session.user_id()).await?;
    for request in requests_as_auditor {
        let project = get_project(&client, &request.project_id).await?;

        views.push(request.to_view(project.name));
    }

    let audits_as_auditor = audits_repo.find_by_auditor(session.user_id()).await?;
    for audit in audits_as_auditor {
        let project = get_project(&client, &audit.project_id).await?;

        views.push(audit.to_view(project.name));
    }

    views.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    Ok(HttpResponse::Ok().json(views.into_iter().map(|a| a.stringify()).collect::<Vec<_>>()))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct AllAuditsQuery {
    tags: String,
    skip: u32,
    limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AllAuditsResponse {
    auditors: Vec<Audit<String>>,
}

#[utoipa::path(
    params(
        AllAuditsQuery
    ),
    responses(
        (status = 200, body = AllAuditsResponse)
    )
)]
#[get("/api/audits/all")]
pub async fn get_audits(
    repo: web::Data<AuditRepo>,
    query: web::Query<AllAuditsQuery>,
) -> Result<HttpResponse> {
    let tags = query.tags.split(',').map(ToString::to_string).collect();
    let auditors = repo.find_by_tags(tags, query.skip, query.limit).await?;
    Ok(HttpResponse::Ok().json(AllAuditsResponse {
        auditors: auditors.into_iter().map(Audit::stringify).collect(),
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PatchAuditRequest {
    pub id: String,
    pub status: Option<String>,
    pub report: Option<String>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),

    request_body(
        content = PatchAuditRequest
    ),
    responses(
        (status = 200, body = AllAuditsResponse)
    )
)]
#[patch("/api/audit")]
pub async fn patch_audit(
    req: HttpRequest,
    Json(data): web::Json<PatchAuditRequest>,
    repo: web::Data<AuditRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let _session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let audit = repo.delete(&data.id.parse().unwrap()).await?;

    let Some(audit) = audit else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };

    let mut audit = audit.clone();
    audit.status = data.status.unwrap_or(audit.status);
    audit.report = data.report.and(audit.report);

    repo.create(&audit).await?;

    Ok(HttpResponse::Ok().json(audit.stringify()))
}

#[utoipa::path(
    responses(
        (status = 200, body = Project<String>)
    )
)]
#[get("/api/audit/by_id/{id}")]
pub async fn audit_by_id(
    id: web::Path<String>,
    repo: web::Data<AuditRepo>,
) -> Result<HttpResponse> {
    let Some(audit) = repo.find(id.parse().unwrap()).await? else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };
    Ok(HttpResponse::Ok().json(audit.stringify()))
}
