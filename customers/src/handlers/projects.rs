use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, post, patch, delete, get, web::{self, Json}};
use common::{auth_session::get_auth_session, entities::project::Project};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::{error::{Result, Error, OuterError}, repositories::{project::ProjectRepository}};



#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostProjectRequest {
    name: String,
    description: String,
    git_url: String,
    git_folders: HashMap<String, String>,
    tags: Vec<String>,
    status: String,
}

#[utoipa::path(
    params(
        ("data" = PostProjectRequest,)
    ),
    responses(
        (status = 200, body = Project)
    )
)]
#[post("/api/projects")]
pub async fn post_project(
    req: HttpRequest, 
    Json(data): web::Json<PostProjectRequest>, 
    repo: web::Data<ProjectRepository>
) -> Result<web::Json<Project>> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let project = Project {
        id: ObjectId::new(),
        customer_id: session.user_id(),
        description: data.description,
        git_folders: data.git_folders,
        git_url: data.git_url,
        name: data.name,
        status: data.status,
        tags: data.tags,
    };

    repo.create(&project).await?;

    Ok(web::Json(project))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchProjectRequest {
    project_id: String,
    name: Option<String>,
    description: Option<String>,
    git_url: Option<String>,
    git_folders: Option<HashMap<String, String>>,
    tags: Option<Vec<String>>,
    status: Option<String>,
}

#[utoipa::path(
    params(
        ("data" = PatchProjectRequest,)
    ),
    responses(
        (status = 200, body = Project)
    )
)]
#[patch("/api/projects")]
pub async fn patch_project(
    req: HttpRequest, 
    web::Json(data): web::Json<PatchProjectRequest>, 
    repo: web::Data<ProjectRepository>
) -> Result<web::Json<Project>> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(mut project) = repo.delete(session.user_id()).await? else {
        return Err(Error::Outer(OuterError::ProjectNotFound))
    };

    if let Some(name) = data.name {
        project.name = name;
    }

    if let Some(description) = data.description {
        project.description = description;
    }

    if let Some(git_url) = data.git_url {
        project.git_url = git_url;
    }

    if let Some(tags) = data.tags {
        project.tags = tags;
    }

    if let Some(git_folders) = data.git_folders {
        project.git_folders = git_folders;
    }

    if let Some(status) = data.status {
        project.status = status;
    }

    repo.delete(session.user_id()).await.unwrap();
    repo.create(&project).await?;

    Ok(web::Json(project))
}


#[utoipa::path(
    responses(
        (status = 200, body = Project)
    )
)]
#[delete("/api/projects")]
pub async fn delete_project(req: HttpRequest, repo: web::Data<ProjectRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(project) = repo.delete(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: project not found
    };

    Ok(HttpResponse::Ok().json(project))
}

#[utoipa::path(
    responses(
        (status = 200, body = Project)
    )
)]
#[get("/api/projects/project")]
pub async fn get_project(req: HttpRequest, repo: web::Data<ProjectRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(Project) = repo.find(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: project not found
    };
    Ok(HttpResponse::Ok().json(Project))
}
