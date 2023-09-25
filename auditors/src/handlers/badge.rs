use actix_web::{
    delete, patch, post,
    web::{self, Json},
    HttpResponse,
};

use crate::service::badge::{BadgeService, CreateBadge};
use common::{
    context::Context,
    entities::{
        auditor::PublicAuditor,
        badge::{Badge, PublicBadge},
    },
    error,
};

#[post("/api/badge")]
pub async fn post_badge(
    context: Context,
    Json(data): web::Json<CreateBadge>,
) -> error::Result<Json<Badge<String>>> {
    Ok(Json(BadgeService::new(context).create(data).await?))
}

#[patch("/api/badge/substitute/{badge_id}/{user_id}")]
pub async fn substitute(
    context: Context,
    ids: web::Path<(String, String)>,
) -> error::Result<HttpResponse> {
    let (badge_id, user_id) = ids.into_inner();

    BadgeService::new(context)
        .substitute(badge_id.parse()?, user_id.parse()?)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[patch("/api/badge/substitute/run/{code}")]
pub async fn substitute_run(
    context: Context,
    ids: web::Path<String>,
) -> error::Result<Json<PublicAuditor>> {
    let code = ids.into_inner();
    Ok(Json(BadgeService::new(context).substitute_run(code).await?))
}

#[delete("/api/badge/delete/{id}")]
pub async fn delete(context: Context, id: web::Path<String>) -> error::Result<Json<PublicBadge>> {
    Ok(Json(BadgeService::new(context).delete(id.parse()?).await?))
}
