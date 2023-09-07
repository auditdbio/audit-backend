use actix_web::{
    get, post,
    web::{self, Json},
};
use common::{context::Context, error};

use crate::service::codes::CodeService;

#[post("/api/code/{email}")]
pub async fn post_code(context: Context, email: web::Path<String>) -> error::Result<String> {
    CodeService::new(context).create(email.into_inner()).await
}

#[get("/api/code/{code}")]
pub async fn check_code(context: Context, code: web::Path<String>) -> error::Result<Json<bool>> {
    Ok(Json(
        CodeService::new(context).check(code.into_inner()).await?,
    ))
}
