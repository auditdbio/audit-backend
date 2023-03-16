use actix_web::{get, web, HttpResponse};
use common::auth_session::AuthSessionManager;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct SearchQuery {
    pub query: String,
    pub tags: Vec<String>,
    pub page: u32,
    pub per_page: u32,
    pub tax_rate: String,
    pub time_from: String,
    pub time_to: String,
    pub ready_to_wait: Option<bool>,
    pub sort_by: String,
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
    _query: web::Query<SearchQuery>,
    _manager: web::Data<AuthSessionManager>,
) -> HttpResponse {
    HttpResponse::Ok().body("Hello, world!")
}
