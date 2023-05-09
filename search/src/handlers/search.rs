use actix_web::{
    get, post,
    web::{self, Json},
    HttpResponse,
};
use common::{context::Context, error};
use mongodb::bson::Document;

use crate::{
    repositories::search::SearchRepo,
    service::search::{SearchInsertRequest, SearchQuery, SearchService},
};

#[post("/api/search/insert")]
pub async fn insert(
    Json(data): Json<SearchInsertRequest>,
    context: Context,
    search_repo: web::Data<SearchRepo>,
) -> error::Result<HttpResponse> {
    SearchService::new(search_repo, context)
        .insert(data)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/api/search")]
pub async fn search(
    query: web::Query<SearchQuery>,
    repo: web::Data<SearchRepo>,
    context: Context,
) -> error::Result<Json<Vec<Document>>> {
    let results = SearchService::new(repo, context)
        .search(query.into_inner())
        .await?;
    Ok(Json(results))
}
