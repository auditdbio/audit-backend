use std::collections::HashMap;
use actix_multipart::Multipart;
use actix_web::{
    post,
    web::{Json, Path, Query},
};

use common::{
    api::report::PublicReport,
    context::GeneralContext,
    error,
};

use crate::service::report::{self, VerifyReportResponse};

#[post("/report/{audit_id}")]
pub async fn create_report(
    context: GeneralContext,
    audit_id: Path<String>,
    query: Query<HashMap<String, String>>,
) -> error::Result<Json<PublicReport>> {
    let code = query.get("code");
    Ok(Json(
        report::create_report(context, audit_id.into_inner(), code).await?,
    ))
}

#[post("/report/{audit_id}/verify")]
pub async fn verify_report(
    context: GeneralContext,
    audit_id: Path<String>,
    payload: Multipart,
) -> error::Result<Json<VerifyReportResponse>> {
    Ok(Json(
        report::verify_report(context, audit_id.into_inner(), payload).await?,
    ))
}