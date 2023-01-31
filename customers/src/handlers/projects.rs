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
    repositories::project::ProjectRepository,
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
    repo: web::Data<ProjectRepository>,
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
    repo: web::Data<ProjectRepository>,
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

    if let Some(scope) = data.scope {
        project.scope = scope;
    }

    if let Some(tags) = data.tags {
        project.tags = tags;
    }

    if let Some(status) = data.status {
        project.status = status;
    }

    repo.delete(session.user_id()).await.unwrap();
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
    repo: web::Data<ProjectRepository>,
) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(project) = repo.delete(session.user_id()).await? else {
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
    repo: web::Data<ProjectRepository>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(project) = repo.find(&id).await? else {
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
        ("Authorization" = String, Header,  description = "Bearer token"),
        AllProjectsQuery,
    ),
    responses(
        (status = 200, body = AllProjectsResponse)
    )
)]
#[get("/api/projects/all/{tags}")]
pub async fn get_projects(
    req: HttpRequest,
    repo: web::Data<ProjectRepository>,
    query: web::Query<AllProjectsQuery>,
) -> Result<HttpResponse> {
    let _session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let tags = query.tags.split(",").map(ToString::to_string).collect();

    let projects = repo.request_with_tags(tags).await?;

    Ok(HttpResponse::Ok().json(projects))
}

#[cfg(test)]
mod tests {
    use std::{env, collections::HashMap};

    use actix_cors::Cors;
    use actix_web::{App, web::{self, service}, test};
    use common::entities::role::Role;
    use mongodb::bson::oid::ObjectId;

    use crate::{repositories::{project::{ProjectRepository}}, post_project, get_views, patch_project, delete_project, get_project};
    use super::{PostProjectRequest, PatchProjectRequest};

    #[actix_web::test]
    async fn test_post_project() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let project_repo = ProjectRepository::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(project_repo.clone()))
            .service(post_project);

        let service = test::init_service(app).await;
        
        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostProjectRequest {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                about: "I am a test project".to_string(),
                tags: vec!["test".to_string()],
                contacts: HashMap::new(),
                company: "Test Company".to_string(),
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success())
    }


    #[actix_web::test]
    async fn test_patch_project() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let project_repo = ProjectRepository::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(project_repo.clone()))
            .service(post_project)
            .service(get_views)
            .service(patch_project);

        let service = test::init_service(app).await;

        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostProjectRequest {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                about: "I am a test project".to_string(),
                tags: vec!["test".to_string()],
                contacts: HashMap::new(),
                company: "Test Company".to_string(),
               
            })
            .to_request();

        let response = test::call_service(&service, req).await;
        assert!(response.status().is_success());

        let req = test::TestRequest::patch()
            .uri("/api/requests")
            .set_json(&PatchProjectRequest {
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
    async fn test_delete_project() {
        let mongo_uri = env::var("MONGOURI_TEST").unwrap();

        let project_repo = ProjectRepository::new(mongo_uri.clone()).await;
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();
        let app = App::new()
            .wrap(cors)
            .app_data(web::Data::new(project_repo.clone()))
            .service(post_project)
            .service(get_project)
            .service(delete_project);

        let service = test::init_service(app).await;

        let req = test::TestRequest::post()
            .uri("/api/requests")
            .set_json(&PostProjectRequest {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                about: "I am a test project".to_string(),
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
