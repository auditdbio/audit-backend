use actix_web::{
    post,
    web::{Json, Path},
};
use common::{api::report::PublicReport, context::GeneralContext, error};

#[post("/report/{id}")]
pub async fn create_report(
    context: GeneralContext,
    id: Path<String>,
) -> error::Result<Json<PublicReport>> {
    Ok(Json(
        crate::service::report::create_report(context, id.into_inner()).await?,
    ))
}
