use actix_web::{
    delete, get,
    web::{self},
    HttpRequest, HttpResponse,
};
use awc::Client;
use chrono::Utc;
use common::{
    auth_session::get_auth_session,
    entities::{audit::Audit, audit_request::AuditRequest, project::Project, view::View},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
        content = AuditRequest
    ),
    responses(
        (status = 200, body = Audit)
    )
)]
#[get("/api/audit")]
pub async fn post_audit(
    req: HttpRequest,
    web::Json(request): web::Json<AuditRequest>,
    repo: web::Data<AuditRepo>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(price) = request.price else {
        return Ok(HttpResponse::BadRequest().body("Price is required"));
    };

    let audit = Audit {
        id: ObjectId::new(),
        customer_id: request.customer_id,
        auditor_id: request.auditor_id,
        project_id: request.project_id,
        status: "pending".to_string(),
        last_modified: Utc::now().naive_utc(),
        auditor_contacts: request.auditor_contacts,
        customer_contacts: request.customer_contacts,
        price: price,
        report_link: None,
        time_frame: request.time_frame,
        scope: request.scope,
    };

    repo.create(&audit).await?;

    Ok(HttpResponse::Ok().json(audit))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Audit)
    )
)]
#[get("/api/audit/{id}")]
pub async fn get_audits(
    req: HttpRequest,
    id: web::Path<ObjectId>,
    repo: web::Data<AuditRepo>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let audits = repo.find(&id).await?;

    Ok(HttpResponse::Ok().json(audits))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Audit)
    )
)]
#[delete("/api/audit/{id}")]
pub async fn delete_audit(
    req: HttpRequest,
    id: web::Path<ObjectId>,
    repo: web::Data<AuditRepo>,
    closed_repo: web::Data<ClosedAuditRepo>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(audit) = repo.delete(&id).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    closed_repo.create(&audit).await?;

    Ok(HttpResponse::Ok().json(audit))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

#[cfg(test)]
mod tests {
    use std::{env, collections::HashMap};

    use actix_cors::Cors;
    use actix_web::{App, web::{self, service}, test};
    use common::entities::role::Role;
    use mongodb::bson::oid::ObjectId;

    use crate::{repositories::{audit::AuditRepo, audit::AuditRepo}, post_audit, get_views, patch_audit, delete_audit};
    use super::{PostAuditRequestRequest, PatchAuditRequestRequest};
    use crate::handlers::audit::GetViewsResponse;

    #[actix_web::test]
    async fn test_post_audit() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let audit_repo = AuditRepo::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(audit_repo.clone()))
            .service(post_audit);

        let service = test::init_service(app).await;
        
        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostAuditRequestRequest {
                customer_id: ObjectId::new(),
                auditor_id: ObjectId::new(),
                project_id: ObjectId::new(),
                auditor_contacts: HashMap::new(),
                customer_contacts: HashMap::new(),
                scope: vec![],
                price: None,
                price_range: None,
                time_frame: String::new(),
                opener: Role::Customer,
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success())
    }


    #[actix_web::test]
    async fn test_patch_audit() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let audit_repo = AuditRepo::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(audit_repo.clone()))
            .service(post_audit)
            .service(get_views)
            .service(patch_audit);

        let service = test::init_service(app).await;

        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostAuditRequestRequest {
                customer_id: ObjectId::new(),
                auditor_id: ObjectId::new(),
                project_id: ObjectId::new(),
                auditor_contacts: HashMap::new(),
                customer_contacts: HashMap::new(),
                scope: vec![],
                price: None,
                price_range: None,
                time_frame: String::new(),
                opener: Role::Customer,
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success());

        let views: GetViewsResponse = test::call_and_read_body_json(&service, test::TestRequest::get().uri("/api/requests/views").to_request()).await;
        let view_id = views.views.first().unwrap().id;


        let req = test::TestRequest::patch()
            .uri("/api/requests")
            .set_json(&PatchAuditRequestRequest {
                id: view_id,
                auditor_contacts: None,
                customer_contacts: None,
                scope: None,
                time_frame: None,
                price: None,
                price_range: None,
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success())

    }

    #[actix_web::test]
    async fn test_delete_audit() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let audit_repo = AuditRepo::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(audit_repo.clone()))
            .service(post_audit)
            .service(get_views)
            .service(delete_audit);

        let service = test::init_service(app).await;

        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostAuditRequestRequest {
                customer_id: ObjectId::new(),
                auditor_id: ObjectId::new(),
                project_id: ObjectId::new(),
                auditor_contacts: HashMap::new(),
                customer_contacts: HashMap::new(),
                scope: vec![],
                price: None,
                price_range: None,
                time_frame: String::new(),
                opener: Role::Customer,
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success());

        let views: GetViewsResponse = test::call_and_read_body_json(&service, test::TestRequest::get().uri("/api/requests/views").to_request()).await;
        let view_id = views.views.first().unwrap().id;


        let req = test::TestRequest::delete()
            .uri("/api/requests")
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success())

    }
}

