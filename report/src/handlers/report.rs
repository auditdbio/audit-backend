use actix_web::{post, web::Json};
use common::{api::audits::PublicAudit, context::Context, error};

use serde::{Deserialize, Serialize};

use crate::service::report::PublicReport;

#[derive(Serialize, Deserialize)]
pub struct CreateReport {
    pub markdown: String,
}

#[derive(Serialize, Deserialize)]
pub struct Report {
    pub latex: String,
}

#[post("/api/report")]
pub async fn create_report(
    context: Context,
    Json(payload): Json<PublicAudit>,
) -> error::Result<Json<PublicReport>> {
    Ok(Json(
        crate::service::report::create_report(context, payload).await?,
    ))
}
