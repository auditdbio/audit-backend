use actix_web::{
    get,
    web::{self, Json},
};
use common::{context::Context, error};
use mongodb::bson::Document;

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
