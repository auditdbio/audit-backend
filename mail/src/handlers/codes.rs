use actix_web::{
    get, post,
    web::{self, Json},
};
use common::{context::GeneralContext, error};

use crate::service::codes::CodeService;

#[post("/api/code/{payload}")]
pub async fn post_code(
    context: GeneralContext,
    path: web::Path<String>,
) -> error::Result<Json<String>> {
    CodeService::new(context)
        .create(path.into_inner())
        .await
        .map(Json)
}

#[get("/api/code/{code}")]
pub async fn check_code(
    context: GeneralContext,
    code: web::Path<String>,
) -> error::Result<Json<Option<String>>> {
    Ok(Json(
        CodeService::new(context).check(code.into_inner()).await?,
    ))
}
