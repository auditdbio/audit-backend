use actix_web::{
    get, post,
    web::{self, Json},
};
use common::{context::Context, error};

use crate::service::codes::CodeService;

#[post("/api/code/{payload}")]
pub async fn post_code(context: Context, path: web::Path<String>) -> error::Result<String> {
    CodeService::new(context).create(path.into_inner()).await
}

#[get("/api/code/{code}")]
pub async fn check_code(
    context: Context,
    code: web::Path<String>,
) -> error::Result<Json<Option<String>>> {
    Ok(Json(
        CodeService::new(context).check(code.into_inner()).await?,
    ))
}
