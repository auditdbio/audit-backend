use actix_web::{
    delete, get, post,
    web::{self, Json},
    HttpResponse,
};
use common::{context::GeneralContext, error};

use crate::{
    repositories::{elasticsearch::ElasticSearchResult, search::SearchRepo},
    service::search::{SearchInsertRequest, SearchQuery, SearchResult, SearchService},
};

#[post("/search/insert")]
pub async fn insert(
    Json(data): Json<SearchInsertRequest>,
    context: GeneralContext,
    search_repo: web::Data<SearchRepo>,
) -> error::Result<HttpResponse> {
    SearchService::new(search_repo, context)
        .insert(data)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/search")]
pub async fn mongo_search(
    query: web::Query<SearchQuery>,
    repo: web::Data<SearchRepo>,
    context: GeneralContext,
) -> error::Result<Json<SearchResult>> {
    let results = SearchService::new(repo, context)
        .search(query.into_inner())
        .await?;
    Ok(Json(results))
}

#[get("/search_new")]
pub async fn search(
    query: web::Query<SearchQuery>,
    repo: web::Data<SearchRepo>,
    context: GeneralContext,
) -> error::Result<Json<ElasticSearchResult>> {
    let results = SearchService::new(repo, context)
        .elasitc_search(query.into_inner())
        .await?;
    Ok(Json(results))
}

#[delete("/search/{id}")]
pub async fn delete(
    id: web::Path<String>,
    repo: web::Data<SearchRepo>,
    context: GeneralContext,
) -> error::Result<HttpResponse> {
    SearchService::new(repo, context)
        .delete(id.parse()?)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
