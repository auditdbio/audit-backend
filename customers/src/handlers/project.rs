use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use common::{context::Context, entities::project::PublicProject, error};
use serde_json::json;

use crate::service::{
    project::{CreateProject, ProjectService, ProjectChange},
};

#[post("/api/project")]
pub async fn post_project(
    context: Context,
    Json(data): Json<CreateProject>,
) -> error::Result<Json<PublicProject>> {
    Ok(Json(ProjectService::new(context).create(data).await?))
}

#[get("/api/project/{id}")]
pub async fn get_project(context: Context, id: web::Path<String>) -> error::Result<HttpResponse> {
    let res = ProjectService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/project/my_project")]
pub async fn my_project(context: Context) -> error::Result<HttpResponse> {
    let res = ProjectService::new(context).my_projects().await?;
    Ok(HttpResponse::Ok().json(res))
}

#[patch("/api/project/{id}")]
pub async fn patch_project(
    context: Context,
    id: web::Path<String>,
    Json(data): Json<ProjectChange>,
) -> error::Result<Json<PublicProject>> {
    Ok(Json(
        ProjectService::new(context)
            .change(id.parse()?, data)
            .await?,
    ))
}

#[delete("/api/customer/{id}")]
pub async fn delete_project(
    context: Context,
    id: web::Path<String>,
) -> error::Result<Json<PublicProject>> {
    Ok(Json(
        ProjectService::new(context).delete(id.parse()?).await?,
    ))
}
