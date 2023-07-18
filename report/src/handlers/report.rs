use actix_web::{
    post,
    web::{Json, Path},
};
use common::{context::Context, error};

use crate::service::report::PublicReport;

#[post("/api/report/{id}")]
pub async fn create_report(
    context: Context,
    id: Path<String>,
) -> error::Result<Json<PublicReport>> {
    Ok(Json(
        crate::service::report::create_report(context, id.into_inner()).await?,
    ))
}
