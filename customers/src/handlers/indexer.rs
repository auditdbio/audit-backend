use actix_web::{
    get, post,
    web::{self, Json},
    HttpResponse,
};
use common::{
    context::GeneralContext,
    entities::{customer::PublicCustomer, project::PublicProject},
    error,
};
use mongodb::bson::{oid::ObjectId, Document};

use crate::service::indexer::IndexerService;

#[get("/customer/data/{since}")]
pub async fn provide_customer_data(
    context: GeneralContext,
    since: web::Path<i64>,
) -> error::Result<Json<Vec<Document>>> {
    Ok(Json(
        IndexerService::new(context)
            .index_customer(since.into_inner())
            .await?,
    ))
}

#[get("/project/data/{since}")]
pub async fn provide_project_data(
    context: GeneralContext,
    since: web::Path<i64>,
) -> error::Result<Json<Vec<Document>>> {
    Ok(Json(
        IndexerService::new(context)
            .index_project(since.into_inner())
            .await?,
    ))
}

#[post("/customer/data")]
pub async fn get_customer_data(
    context: GeneralContext,
    Json(ids): web::Json<Vec<ObjectId>>,
) -> error::Result<Json<Vec<PublicCustomer>>> {
    Ok(Json(
        IndexerService::new(context).find_customers(ids).await?,
    ))
}

#[post("/project/data")]
pub async fn get_project_data(
    context: GeneralContext,
    Json(ids): web::Json<Vec<ObjectId>>,
) -> error::Result<Json<Vec<PublicProject>>> {
    Ok(Json(IndexerService::new(context).find_projects(ids).await?))
}

#[get("/customers/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
