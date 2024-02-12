use actix_web::{
    get, post,
    web::{self, Json},
    HttpResponse,
};
use common::{
    context::GeneralContext,
    entities::{auditor::PublicAuditor, badge::PublicBadge},
    error,
};
use mongodb::bson::{oid::ObjectId, Document};

use crate::service::indexer::IndexerService;

#[get("/auditor/data/{since}")]
pub async fn provide_auditor_data(
    context: GeneralContext,
    since: web::Path<i64>,
) -> error::Result<Json<Vec<Document>>> {
    Ok(Json(
        IndexerService::new(context)
            .index_auditor(since.into_inner())
            .await?,
    ))
}

#[post("/auditor/data")]
pub async fn get_auditor_data(
    context: GeneralContext,
    Json(ids): web::Json<Vec<ObjectId>>,
) -> error::Result<Json<Vec<PublicAuditor>>> {
    Ok(Json(IndexerService::new(context).find_auditors(ids).await?))
}

#[get("/badge/data/{since}")]
pub async fn provide_badges_data(
    context: GeneralContext,
    since: web::Path<i64>,
) -> error::Result<Json<Vec<Document>>> {
    Ok(Json(
        IndexerService::new(context)
            .index_badges(since.into_inner())
            .await?,
    ))
}

#[post("/badge/data")]
pub async fn get_badges_data(
    context: GeneralContext,
    Json(ids): web::Json<Vec<ObjectId>>,
) -> error::Result<Json<Vec<PublicBadge>>> {
    Ok(Json(IndexerService::new(context).find_badges(ids).await?))
}

#[get("/auditors/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}
