use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{
    auth_session::{AuthSessionManager, SessionManager},
    entities::project::Project,
};
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
    manager: web::Data<AuthSessionManager>,
) -> Result<web::Json<Project>> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

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
    project_id: ObjectId,
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
    manager: web::Data<AuthSessionManager>,
) -> Result<web::Json<Project>> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

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
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

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
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let _session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

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

#[cfg(test)]
mod tests {

    use actix_web::test::{self, init_service};
    use common::auth_session::AuthSession;
    use mongodb::bson::oid::ObjectId;

    use crate::{create_test_app, PatchProjectRequest, PostProjectRequest};

    #[actix_web::test]
    async fn test_post_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
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
            exp: 100000000,
        };
        let test_project_id = ObjectId::new();
        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::patch()
            .uri("/api/customer")
            .set_json(&PatchProjectRequest {
                project_id: test_project_id,
                name: Some("Test".to_string()),
                description: Some("Test".to_string()),
                scope: Some(vec!["Test".to_string()]),
                tags: Some(vec!["Test".to_string()]),
                status: Some("Test".to_string()),
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
