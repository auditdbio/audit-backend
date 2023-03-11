use std::collections::HashMap;

use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{
    auth_session::{AuthSessionManager, SessionManager},
    entities::auditor::Auditor,
};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{error::Result, repositories::auditor::AuditorRepo};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostAuditorRequest {
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub free_at: String,
    pub tags: Vec<String>,
    pub contacts: HashMap<String, String>,
    pub tax: String,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PostAuditorRequest
    ),
    responses(
        (status = 200, body = Auditor<String>)
    )
)]
#[post("/api/auditors")]
pub async fn post_auditor(
    req: HttpRequest,
    Json(data): web::Json<PostAuditorRequest>,
    repo: web::Data<AuditorRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let auditor = Auditor {
        user_id: session.user_id(),
        avatar: data.avatar,
        first_name: data.first_name,
        last_name: data.last_name,
        about: data.about,
        company: data.company,
        free_at: data.free_at,
        contacts: data.contacts,
        tags: data.tags,
        tax: data.tax,
    };

    if !repo.create(&auditor).await? {
        return Ok(HttpResponse::BadRequest().finish());
    }
    Ok(HttpResponse::Ok().json(auditor.stringify()))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Auditor<String>)
    )
)]
#[get("/api/auditors")]
pub async fn get_auditor(
    req: HttpRequest,
    repo: web::Data<AuditorRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.find(session.user_id()).await? else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };

    Ok(HttpResponse::Ok().json(auditor.stringify()))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchAuditorRequest {
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    tags: Option<Vec<String>>,
    contacts: Option<HashMap<String, String>>,
    tax: Option<String>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PatchCustomerRequest
    ),
    responses(
        (status = 200, body = Auditor<String>)
    )
)]
#[patch("/api/auditors")]
pub async fn patch_auditor(
    req: HttpRequest,
    web::Json(data): web::Json<PatchAuditorRequest>,
    repo: web::Data<AuditorRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(mut auditor ) = repo.delete(&session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().body("Error: no auditor instance for this user"));
    };

    if let Some(first_name) = data.first_name {
        auditor.first_name = first_name;
    }

    if let Some(last_name) = data.last_name {
        auditor.last_name = last_name;
    }

    if let Some(about) = data.about {
        auditor.about = about;
    }

    if let Some(tags) = data.tags {
        auditor.tags = tags;
    }

    if let Some(contacts) = data.contacts {
        auditor.contacts = contacts;
    }

    if let Some(tax) = data.tax {
        auditor.tax = tax;
    }

    repo.create(&auditor).await?;

    Ok(HttpResponse::Ok().json(auditor.stringify()))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Auditor<String>)
    )
)]
#[delete("/api/auditors")]
pub async fn delete_auditor(
    req: HttpRequest,
    repo: web::Data<AuditorRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.delete(&session.user_id()).await? else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };
    Ok(HttpResponse::Ok().json(auditor.stringify()))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct AllAuditorsQuery {
    tags: String,
    skip: u32,
    limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AllAuditorsResponse {
    auditors: Vec<Auditor<String>>,
}

#[utoipa::path(
    params(
        AllAuditorsQuery
    ),
    responses(
        (status = 200, body = AllAuditorsResponse)
    )
)]
#[get("/api/auditors/all")]
pub async fn get_auditors(
    repo: web::Data<AuditorRepo>,
    query: web::Query<AllAuditorsQuery>,
) -> Result<HttpResponse> {
    let tags = query.tags.split(',').map(ToString::to_string).collect();
    let auditors = repo.find_by_tags(tags, query.skip, query.limit).await?;
    Ok(HttpResponse::Ok().json(AllAuditorsResponse {
        auditors: auditors.into_iter().map(Auditor::stringify).collect(),
    }))
}

#[cfg(test)]
mod tests {
    use actix_web::test::{self, init_service};
    use common::auth_session::AuthSession;
    use mongodb::bson::oid::ObjectId;

    use crate::{create_test_app, PostAuditorRequest};

    #[actix_web::test]
    async fn test_post_auditor() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            exp: 100000000,
        };

        let mut app = init_service(create_test_app(test_user)).await;

        let req = test::TestRequest::post()
            .uri("/api/auditors")
            .set_json(&PostAuditorRequest {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                avatar: "https://test.com".to_string(),
                about: "About me".to_string(),
                company: "Company".to_string(),
                free_at: "2020-01-01".to_string(),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                contacts: vec![("email".to_string(), "test@test.com".to_string())]
                    .into_iter()
                    .collect(),
                tax: "200".to_string(),
            })
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());
    }
}

#[utoipa::path(
    responses(
        (status = 200, body = Auditor<String>)
    )
)]
#[get("/api/auditors/by_id/{id}")]
pub async fn auditor_by_id(
    id: web::Path<String>,
    repo: web::Data<AuditorRepo>,
) -> Result<HttpResponse> {
    let Some(auditor) = repo.find(id.parse().unwrap()).await? else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };
    Ok(HttpResponse::Ok().json(auditor.stringify()))
}

