use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, post, patch, delete, get, web::{self, Json}};
use common::get_auth_session;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::{error::Result, repositories::{customer::CustomerRepository, project::{ProjectModel, ProjectRepository}}};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostCustomerRequest {
    name: String,
    description: String,
    git_url: String,
    git_folders: HashMap<String, String>,
    tags: Vec<String>,
    status: String,
}

#[post("/api/projects")]
pub async fn post_project(req: HttpRequest, Json(data): web::Json<PostCustomerRequest>, repo: web::Data<ProjectRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let project = ProjectModel {
        id: ObjectId::new(),
        customer_id: session.user_id(),
        description: data.description,
        git_folders: data.git_folders,
        git_url: data.git_url,
        name: data.name,
        status: data.status,
        tags: data.tags,
    };

    if !repo.create(project).await? {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: customer entity already exits
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchcustomerRequest {
    project_id: String,
    name: Option<String>,
    description: Option<String>,
    git_url: Option<String>,
    git_folders: Option<HashMap<String, String>>,
    tags: Option<Vec<String>>,
    status: Option<String>,
}

#[patch("/api/projects")]
pub async fn patch_project(req: HttpRequest, web::Json(data): web::Json<PatchcustomerRequest>, repo: web::Data<ProjectRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(mut project) = repo.delete(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: project not found
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
    repo.create(project).await?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/api/projects")]
pub async fn delete_project(req: HttpRequest, repo: web::Data<CustomerRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(project) = repo.delete(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: project not found
    };

    Ok(HttpResponse::Ok().json(project))
}

#[get("/api/projects/project")]
pub async fn get_project(req: HttpRequest, repo: web::Data<CustomerRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(customer) = repo.find(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: project not found
    };
    Ok(HttpResponse::Ok().json(customer))
}
