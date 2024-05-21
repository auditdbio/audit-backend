use actix_web::{post, web::Json};
use common::{context::GeneralContext, error};
use serde::{Deserialize, Serialize};

use crate::{repositories::file_repo::CountResult, services::ClocService};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClocRequest {
    pub links: Vec<String>,
}

#[post("/cloc/count")]
pub async fn count(
    context: GeneralContext,
    Json(request): Json<ClocRequest>,
) -> error::Result<Json<CountResult>> {
    Ok(Json(ClocService::new(context).count(request).await?))
}
