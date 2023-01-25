use actix_web::{
    delete, get,
    web::{self},
    HttpRequest, HttpResponse,
};
use awc::Client;
use common::{
    auth_session::get_auth_session,
    entities::{project::Project, view::View},
};
use mongodb::bson::oid::ObjectId;

use crate::{
    contants::CUSTOMERS_SERVICE,
    error::Result,
    repositories::{audit::AuditRepo, audit_request::AuditRequestRepo, closed_audits::ClosedAuditRepo},
};


#[utoipa::path(
    responses(
        (status = 200, body = Auditor)
    )
)]
#[get("/api/audit/{id}")]
pub async fn get_audits(req: HttpRequest, id: web::Path<ObjectId>, repo: web::Data<AuditRepo>) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let audits = repo.find(&id).await?;

    Ok(HttpResponse::Ok().json(audits))
}


#[utoipa::path(
    responses(
        (status = 200, body = Auditor)
    )
)]
#[delete("/api/audit/{id}")]
pub async fn delete_audit(
    req: HttpRequest,
    id: web::Path<ObjectId>,
    repo: web::Data<AuditRepo>,
    closed_repo: web::Data<ClosedAuditRepo>
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(audit) = repo.delete(&id).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    closed_repo.create(&audit).await?;

    Ok(HttpResponse::Ok().json(audit))
}

pub struct GetViewsResponse {
    pub views: Vec<View>,
}

async fn get_project(client: &Client, project_id: &ObjectId) -> Result<Project> {
    let mut res = client
        .get(format!(
            "http://{}/api/project/{}",
            CUSTOMERS_SERVICE, project_id
        ))
        .send()
        .await
        .unwrap();

    let body = res.json::<Project>().await.unwrap();
    Ok(body)
}


#[utoipa::path(
    responses(
        (status = 200, body = Auditor)
    )
)]
#[get("/api/audit/")]
pub async fn get_views(
    req: HttpRequest,
    request_repo: web::Data<AuditRequestRepo>,
    audits_repo: web::Data<AuditRepo>,
) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let mut views = Vec::new();
    let mut client = awc::Client::default();

    client.headers().unwrap().insert(
        "Authorization".parse().unwrap(),
        req.headers().get("Authorization").unwrap().clone(),
    );

    let requests_as_customer = request_repo.find_by_customer(&session.user_id()).await?;
    for request in requests_as_customer {
        let project = get_project(&client, &request.project_id).await?;

        views.push(request.to_view(project.name));
    }

    let requests_as_auditor = request_repo.find_by_auditor(&session.user_id()).await?;
    for request in requests_as_auditor {
        let project = get_project(&client, &request.project_id).await?;

        views.push(request.to_view(project.name));
    }

    let audits_as_auditor = audits_repo.find_by_auditor(&session.user_id()).await?;
    for audit in audits_as_auditor {
        let project = get_project(&client, &audit.project_id).await?;

        views.push(audit.to_view(project.name));
    }

    let audits_as_customer = audits_repo.find_by_customer(&session.user_id()).await?;
    for audit in audits_as_customer {
        let project = get_project(&client, &audit.project_id).await?;

        views.push(audit.to_view(project.name));
    }

    views.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    Ok(HttpResponse::Ok().json(views))
}
