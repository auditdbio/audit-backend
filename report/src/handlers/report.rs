use std::collections::HashMap;
use actix_web::{
    post,
    web::{Json, Path, Query},
};
use common::{api::report::PublicReport, context::GeneralContext, error};

#[post("/report/{id}")]
pub async fn create_report(
    context: GeneralContext,
    id: Path<String>,
    query: Query<HashMap<String, String>>,
) -> error::Result<Json<PublicReport>> {
    let code = query.get("code");
    Ok(Json(
        crate::service::report::create_report(context, id.into_inner(), code).await?,
    ))
}
