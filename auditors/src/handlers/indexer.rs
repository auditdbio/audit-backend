use actix_web::{
    get, post,
    web::{self, Json},
};
use common::{context::Context, entities::auditor::PublicAuditor, error};
use mongodb::bson::{oid::ObjectId, Document};

use crate::service::indexer::IndexerService;

#[get("/api/auditor/data/{since}")]
pub async fn provide_auditor_data(
    context: Context,
    since: web::Path<i64>,
) -> error::Result<Json<Vec<Document>>> {
    Ok(Json(
        IndexerService::new(context)
            .index_auditor(since.into_inner())
            .await?,
    ))
}

#[post("/api/auditor/data")]
pub async fn get_auditor_data(
    context: Context,
    Json(ids): web::Json<Vec<ObjectId>>,
) -> error::Result<Json<Vec<PublicAuditor>>> {
    Ok(Json(IndexerService::new(context).find_auditors(ids).await?))
}
