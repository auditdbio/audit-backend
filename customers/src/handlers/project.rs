use actix_web::{
    delete, get, patch, post,
    web::{self, Json, Path},
    HttpResponse,
};

use common::{context::GeneralContext, entities::project::PublicProject, error};
use serde_json::json;

use crate::service::project::{CreateProject, ProjectChange, ProjectService};

#[post("/api/project")]
pub async fn post_project(
    context: GeneralContext,
    Json(data): Json<CreateProject>,
) -> error::Result<Json<PublicProject>> {
    Ok(Json(ProjectService::new(context).create(data).await?))
}

#[get("/api/project/{id}")]
pub async fn get_project(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let res = ProjectService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/my_project")]
pub async fn my_project(context: GeneralContext) -> error::Result<HttpResponse> {
    let res = ProjectService::new(context).my_projects().await?;
    Ok(HttpResponse::Ok().json(res))
}

#[patch("/api/project/{id}")]
pub async fn patch_project(
    context: GeneralContext,
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
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<Json<PublicProject>> {
    Ok(Json(
        ProjectService::new(context).delete(id.parse()?).await?,
    ))
}

#[post("/api/project/auditor/{id}/{user_id}")]
pub async fn add_auditor(
    context: GeneralContext,
    ids: Path<(String, String)>,
) -> error::Result<HttpResponse> {
    let (id, user_id) = ids.into_inner();
    ProjectService::new(context)
        .add_auditor(id.parse()?, user_id.parse()?)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[delete("/api/project/auditor/{id}/{user_id}")]
pub async fn delete_auditor(
    context: GeneralContext,
    ids: Path<(String, String)>,
) -> error::Result<HttpResponse> {
    let (id, user_id) = ids.into_inner();
    ProjectService::new(context)
        .delete_auditor(id.parse()?, user_id.parse()?)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
