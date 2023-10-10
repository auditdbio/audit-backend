use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use crate::service::badge::{BadgeService, CreateBadge};
use common::{context::Context, entities::badge::Badge, error};

#[post("/api/badge")]
pub async fn post_badge(
    context: Context,
    Json(data): web::Json<CreateBadge>,
) -> error::Result<Json<Badge<String>>> {
    Ok(Json(BadgeService::new(context).create(data).await?))
}

#[patch("/api/badge/merge/{badge_id}")]
pub async fn substitute(context: Context, ids: web::Path<String>) -> error::Result<HttpResponse> {
    let badge_id = ids.into_inner();

    BadgeService::new(context)
        .substitute(badge_id.parse()?)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/api/badge/merge/run/{code}")]
pub async fn substitute_run(
    context: Context,
    ids: web::Path<String>,
) -> error::Result<HttpResponse> {
    let code = ids.into_inner();
    BadgeService::new(context).substitute_run(code).await?;
    Ok(HttpResponse::Ok().finish())
}

#[delete("/api/badge/delete/{id}")]
pub async fn delete(context: Context, id: web::Path<String>) -> error::Result<HttpResponse> {
    BadgeService::new(context).delete(id.parse()?).await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/api/badge/delete/run/{code}")]
pub async fn delete_run(context: Context, code: web::Path<String>) -> error::Result<HttpResponse> {
    BadgeService::new(context).delete_run(code.parse()?).await?;
    Ok(HttpResponse::Ok().finish())
}
