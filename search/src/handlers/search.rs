use actix_web::{get, post, web::{self, Json}, HttpResponse};
use common::auth_session::AuthSessionManager;
use mongodb::bson::Document;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

use crate::repositories::search::SearchRepo;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchInsertRequest {
    documents: Vec<Document>,
}

#[utoipa::path(
    request_body(
        content = SearchInsertRequest,
    ),
    responses(
        (status = 200, body = GetAuditResponse)
    )
)]
#[post("/api/search/insert")]
pub async fn insert_query(
    Json(data): web::Json<SearchInsertRequest>,
    search_repo: web::Data<SearchRepo>,
) -> HttpResponse {
    search_repo.insert(data.documents).await;
    HttpResponse::Ok().finish()
}

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct SearchQuery {
    pub query: String,
    pub tags: Vec<String>,
    pub page: u32,
    pub per_page: u32,
    pub tax_rate_from: usize,
    pub tax_rate_to: usize,
    pub time_from: usize,
    pub time_to: usize,
    pub ready_to_wait: Option<bool>,
    pub sort_by: String,
    pub sort_order: i32,
    pub kind: Option<String>,
}

#[utoipa::path(
    params(
        SearchQuery,
    ),
    responses(
        (status = 200, body = GetAuditResponse)
    )
)]
#[get("/api/search")]
pub async fn search(
    query: web::Query<SearchQuery>,
    _manager: web::Data<AuthSessionManager>,
    repo: web::Data<SearchRepo>
) -> HttpResponse {
    let results = repo.find(query.into_inner()).await;
    HttpResponse::Ok().json(results)
}
