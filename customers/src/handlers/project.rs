use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use common::{context::Context, entities::project::PublicProject, error};
use serde_json::json;

use crate::service::{
    customer::{CustomerChange, CustomerService, PublicCustomer},
    project::{CreateProject, ProjectService},
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
    let res = CustomerService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[patch("/api/project/{id}")]
pub async fn patch_project(
    context: Context,
    id: web::Path<String>,
    Json(data): Json<CustomerChange>,
) -> error::Result<Json<PublicCustomer>> {
    Ok(Json(
        CustomerService::new(context)
            .change(id.parse()?, data)
            .await?,
    ))
}

#[delete("/api/customer/{id}")]
pub async fn delete_project(
    context: Context,
    id: web::Path<String>,
) -> error::Result<Json<PublicCustomer>> {
    Ok(Json(
        CustomerService::new(context).delete(id.parse()?).await?,
    ))
}
