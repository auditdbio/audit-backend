use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
};

use crate::service::bage::{BageService, CreateBage};
use common::{
    context::Context,
    entities::{
        auditor::PublicAuditor,
        bage::{Bage, PublicBage},
    },
    error,
};

#[post("/api/bage")]
pub async fn post_bage(
    context: Context,
    Json(data): web::Json<CreateBage>,
) -> error::Result<Json<Bage<String>>> {
    Ok(Json(BageService::new(context).create(data).await?))
}

#[patch("/api/bage/substitute/{bage_id}/{user_id}")]
pub async fn substitute(
    context: Context,
    ids: web::Path<(String, String)>,
) -> error::Result<Json<PublicAuditor>> {
    let (bage_id, user_id) = ids.into_inner();
    Ok(Json(
        BageService::new(context)
            .substitute(bage_id.parse()?, user_id.parse()?)
            .await?,
    ))
}

#[delete("/api/bage/delete/{id}")]
pub async fn delete(context: Context, id: web::Path<String>) -> error::Result<Json<PublicBage>> {
    Ok(Json(BageService::new(context).delete(id.parse()?).await?))
}
