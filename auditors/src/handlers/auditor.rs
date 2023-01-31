use std::collections::HashMap;

use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{auth_session::get_auth_session, entities::auditor::Auditor};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{error::Result, repositories::auditor::AuditorRepository};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostAuditorRequest {
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub tags: Vec<String>,
    pub contacts: HashMap<String, String>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PostAuditorRequest
    ),
    responses(
        (status = 200, body = Auditor)
    )
)]
#[post("/api/auditors")]
pub async fn post_auditor(
    req: HttpRequest,
    Json(data): web::Json<PostAuditorRequest>,
    repo: web::Data<AuditorRepository>,
) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let auditor = Auditor {
        user_id: session.user_id(),
        first_name: data.first_name,
        last_name: data.last_name,
        about: data.about,
        company: data.company,
        contacts: data.contacts,
        tags: data.tags,
    };

    if !repo.create(&auditor).await? {
        return Ok(HttpResponse::BadRequest().finish());
    }
    Ok(HttpResponse::Ok().json(auditor))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Auditor)
    )
)]
#[get("/api/auditors")]
pub async fn get_auditor(
    req: HttpRequest,
    repo: web::Data<AuditorRepository>,
) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.find(&session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    Ok(HttpResponse::Ok().json(auditor))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchAuditorRequest {
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    tags: Option<Vec<String>>,
    contacts: Option<HashMap<String, String>>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PatchCustomerRequest
    ),
    responses(
        (status = 200, body = Auditor)
    )
)]
#[patch("/api/auditors")]
pub async fn patch_auditor(
    req: HttpRequest,
    web::Json(data): web::Json<PatchAuditorRequest>,
    repo: web::Data<AuditorRepository>,
) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(mut auditor ) = repo.find(&session.user_id()).await? else {
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

    Ok(HttpResponse::Ok().json(auditor))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Auditor)
    )
)]
#[delete("/api/auditors")]
pub async fn delete_auditor(
    req: HttpRequest,
    repo: web::Data<AuditorRepository>,
) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(auditor) = repo.delete(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    Ok(HttpResponse::Ok().json(auditor))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct AllAuditorsQuery {
    tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AllAuditorsResponse {
    auditors: Vec<Auditor>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
        AllAuditorsQuery
    ),
    responses(
        (status = 200, body = AllAuditorsResponse)
    )
)]
#[get("/api/auditors/all")]
pub async fn get_auditors(
    repo: web::Data<AuditorRepository>,
    query: web::Query<AllAuditorsQuery>,
) -> Result<HttpResponse> {
    let tags = query.tags.split(',').map(ToString::to_string).collect();
    let auditors = repo.request_with_tags(tags).await?;
    Ok(HttpResponse::Ok().json(auditors))
}

#[cfg(test)]
mod tests {
    use std::{env, collections::HashMap};

    use actix_cors::Cors;
    use actix_web::{App, web::{self, service}, test};
    use common::entities::role::Role;
    use mongodb::bson::oid::ObjectId;

    use crate::{repositories::{auditor::{AuditorRepository}}, post_auditor, get_views, patch_auditor, delete_auditor, get_auditor};
    use super::{PostAuditorRequest, PatchAuditorRequest};

    #[actix_web::test]
    async fn test_post_auditor() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let auditor_repo = AuditorRepository::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(auditor_repo.clone()))
            .service(post_auditor);

        let service = test::init_service(app).await;
        
        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostAuditorRequest {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                about: "I am a test auditor".to_string(),
                tags: vec!["test".to_string()],
                contacts: HashMap::new(),
                company: "Test Company".to_string(),
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success())
    }


    #[actix_web::test]
    async fn test_patch_auditor() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let auditor_repo = AuditorRepository::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(auditor_repo.clone()))
            .service(post_auditor)
            .service(get_views)
            .service(patch_auditor);

        let service = test::init_service(app).await;

        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostAuditorRequest {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                about: "I am a test auditor".to_string(),
                tags: vec!["test".to_string()],
                contacts: HashMap::new(),
                company: "Test Company".to_string(),
               
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success());

        let req = test::TestRequest::patch()
            .uri("/api/requests")
            .set_json(&PatchAuditorRequest {
                first_name: None,
                last_name: None,
                about: None,
                tags: None,
                contacts: None,
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success())

    }

    #[actix_web::test]
    async fn test_delete_auditor() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let auditor_repo = AuditorRepository::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(auditor_repo.clone()))
            .service(post_auditor)
            .service(get_auditor)
            .service(delete_auditor);

        let service = test::init_service(app).await;

        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostAuditorRequest {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                about: "I am a test auditor".to_string(),
                tags: vec!["test".to_string()],
                contacts: HashMap::new(),
                company: "Test Company".to_string(),
               
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success());

        let req = test::TestRequest::delete()
            .uri("/api/requests")
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success())

    }
}
