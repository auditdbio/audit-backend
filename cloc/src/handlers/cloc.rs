use actix_web::{post, web::Json};
use common::{context::GeneralContext, error};
use serde::{Deserialize, Serialize};

use crate::services::{ClocCount, ClocService};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClocRequest {
    pub author: String,
    pub repo: String,
    pub branch: Option<String>,
    pub commit: Option<String>,
}

#[post("/api/cloc/count")]
pub async fn count(
    context: GeneralContext,
    Json(request): Json<ClocRequest>,
) -> error::Result<Json<ClocCount>> {
    Ok(Json(ClocService::new(context).count(request).await?))
}
