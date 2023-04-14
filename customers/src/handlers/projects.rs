use std::collections::HashMap;

use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use common::{
    auth_session::{AuthSessionManager, SessionManager},
    entities::{project::{Project, PublishOptions}, audit_request::{TimeRange, PriceRange}},
};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    error::{Error, OuterError, Result},
    repositories::project::ProjectRepo,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostProjectRequest {
    name: String,
    description: String,
    scope: Vec<String>,
    tags: Vec<String>,
    status: String,
    publish: bool,
    ready_to_wait: bool,
    creator_contacts: HashMap<String, String>,
    price_range: PriceRange,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PostProjectRequest,
    ),
    responses(
        (status = 200, body = Project<String>)
    )
)]
#[post("/api/projects")]
pub async fn post_project(
    req: HttpRequest,
    Json(data): web::Json<PostProjectRequest>,
    repo: web::Data<ProjectRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<web::Json<Project<String>>> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let project = Project {
        id: ObjectId::new(),
        customer_id: session.user_id(),
        name: data.name,
        description: data.description,
        scope: data.scope,
        tags: data.tags,
        publish_options: PublishOptions {
            publish: data.publish,
            ready_to_wait: data.ready_to_wait,
        },
        status: data.status,
        creator_contacts: data.creator_contacts,
        last_modified: Utc::now().timestamp_micros(),
        price_range: data.price_range,
    };

    repo.create(&project).await?;

    Ok(web::Json(project.stringify()))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchProjectRequest {
    id: String,
    name: Option<String>,
    description: Option<String>,
    scope: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    status: Option<String>,
    creator_contacts: Option<HashMap<String, String>>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PatchProjectRequest,
    ),
    responses(
        (status = 200, body = Project<String>)
    )
)]
#[patch("/api/projects")]
pub async fn patch_project(
    req: HttpRequest,
    web::Json(data): web::Json<PatchProjectRequest>,
    repo: web::Data<ProjectRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<web::Json<Project<String>>> {
    let _session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap
    let id = data.id.parse::<ObjectId>().unwrap();

    let Some(mut project) = repo.delete(&id).await? else {
        return Err(Error::Outer(OuterError::ProjectNotFound))
    };

    if let Some(name) = data.name {
        project.name = name;
    }

    if let Some(description) = data.description {
        project.description = description;
    }

    if let Some(scope) = data.scope {
        project.scope = scope;
    }

    if let Some(tags) = data.tags {
        project.tags = tags;
    }

    if let Some(status) = data.status {
        project.status = status;
    }

    repo.create(&project).await?;

    Ok(web::Json(project.stringify()))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Project<String>)
    )
)]
#[delete("/api/projects")]
pub async fn delete_project(
    req: HttpRequest,
    repo: web::Data<ProjectRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(project) = repo.delete(&session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: project not found
    };

    Ok(HttpResponse::Ok().json(project))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProjectResponse {
    projects: Vec<Project<String>>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = GetProjectResponse)
    )
)]
#[get("/api/projects")]
pub async fn get_project(
    req: HttpRequest,
    repo: web::Data<ProjectRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let projects = repo.find_by_customer(session.user_id()).await?;
    Ok(HttpResponse::Ok().json(GetProjectResponse {
        projects: projects.into_iter().map(Project::stringify).collect(),
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct AllProjectsQuery {
    tags: String,
    skip: u32,
    limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AllProjectsResponse {
    tags: Vec<Project<String>>,
}

#[utoipa::path(
    params(
        AllProjectsQuery,
    ),
    responses(
        (status = 200, body = AllProjectsResponse)
    )
)]
#[get("/api/projects/all")]
pub async fn get_projects(
    repo: web::Data<ProjectRepo>,
    query: web::Query<AllProjectsQuery>,
) -> Result<HttpResponse> {
    let tags = query
        .tags
        .split(",")
        .map(ToString::to_string)
        .filter(|s| !s.is_empty())
        .collect();

    let projects = repo
        .find_by_tags(tags, query.skip, query.limit)
        .await?
        .into_iter()
        .map(Project::stringify)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(projects))
}

#[utoipa::path(
    responses(
        (status = 200, body = Project<String>)
    )
)]
#[get("/api/projects/by_id/{id}")]
pub async fn project_by_id(
    id: web::Path<String>,
    repo: web::Data<ProjectRepo>,
) -> Result<HttpResponse> {
    let Some(project) = repo.find(id.parse().unwrap()).await? else {
        return Ok(HttpResponse::Ok().json(doc!{}));
    };
    Ok(HttpResponse::Ok().json(project.stringify()))
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use actix_web::test::{self, init_service};
    use common::{auth_session::{AuthSession, Role}, entities::audit_request::PriceRange};
    use mongodb::bson::oid::ObjectId;

    use crate::{create_test_app, PatchProjectRequest, PostProjectRequest};

    #[actix_web::test]
    async fn test_post_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            role: Role::User,
            exp: 100000000,
        };
        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::post()
            .uri("/api/customer")
            .set_json(&PostProjectRequest {
                name: "Test".to_string(),
                description: "I'm a test".to_string(),
                scope: vec!["Test".to_string()],
                tags: vec!["Test".to_string()],
                status: "Test".to_string(),
                publish: true,
                ready_to_wait: false,
                creator_contacts: HashMap::new(),
                price_range: PriceRange {
                    begin: 0,
                    end: 100,
                },
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }

    #[actix_web::test]
    async fn test_patch_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            role: Role::User,
            exp: 100000000,
        };
        let test_project_id = ObjectId::new();
        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::patch()
            .uri("/api/customer")
            .set_json(&PatchProjectRequest {
                id: test_project_id.to_hex(),
                name: Some("Test".to_string()),
                description: Some("Test".to_string()),
                scope: Some(vec!["Test".to_string()]),
                tags: Some(vec!["Test".to_string()]),
                status: Some("Test".to_string()),
                creator_contacts: None,
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }

    #[actix_web::test]
    async fn test_delete_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            role: Role::User,
            exp: 100000000,
        };
        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::delete()
            .uri("/api/customer")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            role: Role::User,
            exp: 100000000,
        };
        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/api/customer")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }
}
