use actix_web::{
    get, post,
    web::{Data, Json, Query},
    HttpResponse,
};
use serde::Deserialize;

use crate::{
    error::ServiceResult,
    models::{SearchQuery, SearchResponse},
    search::SearchService,
};

#[derive(Deserialize)]
pub struct SyncQuery {
    force: Option<bool>,
}

#[post("/sync")]
pub async fn sync(
    service: Data<SearchService>,
    query: Query<SyncQuery>,
) -> ServiceResult<HttpResponse> {
    if query.force.unwrap_or(false) {
        service.clear().await?;
    }
    service.sync().await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/cleanup")]
pub async fn cleanup(service: Data<SearchService>) -> ServiceResult<HttpResponse> {
    service.cleanup().await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/search")]
pub async fn search(
    service: Data<SearchService>,
    query: Query<SearchQuery>,
) -> ServiceResult<Json<SearchResponse>> {
    let response = service.search(query.into_inner()).await?;
    Ok(Json(response))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(sync)
        .service(cleanup)
        .service(search);
} 