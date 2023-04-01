use actix_web::{
    get, post,
    web::{self, Json},
    HttpResponse,
};
use chrono::Utc;
use common::auth_session::AuthSessionManager;
use mongodb::bson::Document;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use utoipa::{IntoParams, ToSchema};

use crate::repositories::{search::SearchRepo, since::SinceRepo};

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
    pub tags: String,
    pub page: u32,
    pub per_page: u32,
    pub tax_rate_from: Option<usize>,
    pub tax_rate_to: Option<usize>,
    pub time_from: Option<usize>,
    pub time_to: Option<usize>,
    pub ready_to_wait: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<i32>,
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
    repo: web::Data<SearchRepo>,
) -> HttpResponse {
    let results = repo.find(query.into_inner()).await;
    HttpResponse::Ok().json(results)
}

pub(super) async fn get_data(
    client: &Client,
    resource: &String,
    origin: &String,
    since: i64,
) -> Option<Vec<Document>> {
    let Ok(res) = client
        .get(format!("https://{origin}/data/{resource}/{since}"))
        .send()
        .await else {
        return None;
    };
    let Ok(body) = res.json::<Vec<Document>>().await else {
        return None;
    };
    Some(body)
}

pub async fn fetch_data(since_repo: SinceRepo, search_repo: SearchRepo) {
    let client = Client::new();
    let data = since_repo.get_all().await.unwrap();

    for mut since in data {
        since.since = Utc::now().timestamp_micros();
        let Some(docs) = get_data(&client, &since.resource, &since.service_origin, since.since).await else {
            continue;
        };
        since_repo.update(since).await.unwrap();
        search_repo.insert(docs).await;
    }
}
