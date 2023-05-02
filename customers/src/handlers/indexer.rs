use actix_web::{
    get,
    web::{self, Json}, post,
};
use common::{context::Context, error, entities::{customer::PublicCustomer, project::PublicProject}};
use mongodb::bson::{Document, oid::ObjectId};

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

#[post("/api/customer/data")]
pub async fn get_customer_data(
    context: Context,
    Json(ids): web::Json<Vec<ObjectId>>,
) -> error::Result<Json<Vec<PublicCustomer>>> {
    Ok(Json(
        IndexerService::new(context)
            .find_customers(ids)
            .await?,
    ))
}


#[post("/api/project/data")]
pub async fn get_project_data(
    context: Context,
    Json(ids): web::Json<Vec<ObjectId>>,
) -> error::Result<Json<Vec<PublicProject>>> {
    Ok(Json(
        IndexerService::new(context)
            .find_projects(ids)
            .await?,
    ))
}
