use actix_web::{
    get,
    web::{self, Json},
};
use common::{context::Context, error};
use mongodb::bson::Document;

use crate::service::indexer::IndexerService;

#[get("/api/customer/data/{since}")]
pub async fn provide_customer_data(
    context: Context,
    since: web::Path<i64>,
) -> error::Result<Json<Vec<Document>>> {
    Ok(Json(
        IndexerService::new(context)
            .index_customer(since.into_inner())
            .await?,
    ))
}

#[get("/api/project/data/{since}")]
pub async fn provide_project_data(
    context: Context,
    since: web::Path<i64>,
) -> error::Result<Json<Vec<Document>>> {
    Ok(Json(
        IndexerService::new(context)
            .index_project(since.into_inner())
            .await?,
    ))
}
