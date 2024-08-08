use mongodb::bson::{oid::ObjectId, Document};
use actix_web::{
    get, post,
    web::{Path, Json},
};

use crate::service::indexer::IndexerService;
use common::{
    context::GeneralContext,
    entities::organization::PublicOrganization,
    error,
};

#[get("/organization/data/{since}")]
pub async fn provide_organization_data(
    context: GeneralContext,
    since: Path<i64>,
) -> error::Result<Json<Vec<Document>>> {
    Ok(Json(
        IndexerService::new(context)
            .index_organization(since.into_inner())
            .await?,
    ))
}

#[post("/organization/data")]
pub async fn get_organization_data(
    context: GeneralContext,
    Json(ids): Json<Vec<ObjectId>>,
) -> error::Result<Json<Vec<PublicOrganization>>> {
    Ok(Json(IndexerService::new(context).find_organizations(ids).await?))
}
