use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{auth_session::get_auth_session, entities::project::Project};
use mongodb::bson::oid::ObjectId;
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
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PostProjectRequest,
    ),
    responses(
        (status = 200, body = Project)
    )
)]
#[post("/api/projects")]
pub async fn post_project(
    req: HttpRequest,
    Json(data): web::Json<PostProjectRequest>,
    repo: web::Data<ProjectRepo>,
) -> Result<web::Json<Project>> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let project = Project {
        id: ObjectId::new(),
        customer_id: session.user_id(),
        description: data.description,
        name: data.name,
        status: data.status,
        tags: data.tags,
        scope: data.scope,
    };

    repo.create(&project).await?;

    Ok(web::Json(project))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchProjectRequest {
    project_id: String,
    name: Option<String>,
    description: Option<String>,
    scope: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    status: Option<String>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PatchProjectRequest,
    ),
    responses(
        (status = 200, body = Project)
    )
)]
#[patch("/api/projects")]
pub async fn patch_project(
    req: HttpRequest,
    web::Json(data): web::Json<PatchProjectRequest>,
    repo: web::Data<ProjectRepo>,
) -> Result<web::Json<Project>> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(mut project) = repo.delete(&session.user_id()).await? else {
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

    repo.delete(&session.user_id()).await.unwrap();
    repo.create(&project).await?;

    Ok(web::Json(project))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Project)
    )
)]
#[delete("/api/projects")]
pub async fn delete_project(
    req: HttpRequest,
    repo: web::Data<ProjectRepo>,
) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(project) = repo.delete(&session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: project not found
    };

    Ok(HttpResponse::Ok().json(project))
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Project)
    )
)]
#[get("/api/projects/project/{id}")]
pub async fn get_project(
    req: HttpRequest,
    id: web::Path<ObjectId>,
    repo: web::Data<ProjectRepo>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(project) = repo.find(id.into_inner()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: project not found
    };
    Ok(HttpResponse::Ok().json(project))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct AllProjectsQuery {
    tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AllProjectsResponse {
    tags: Vec<Project>,
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
    let tags = query.tags.split(",").map(ToString::to_string).collect();

    let projects = repo.find_by_tags(tags).await?;

    Ok(HttpResponse::Ok().json(projects))
}
