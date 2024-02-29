use actix_web::{post, web::Json};
use common::{context::GeneralContext, error};
use serde::{Deserialize, Serialize};

use crate::services::ClocService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClocRequest {
    pub links: Vec<(String, String)>,
}

#[post("/cloc/count")]
pub async fn count(
    context: GeneralContext,
    Json(request): Json<ClocRequest>,
) -> error::Result<String> {
    Ok(ClocService::new(context).count(request).await?)
}
