use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use crate::service::badge::{BadgeService, CreateBadge};
use common::{
    context::GeneralContext,
    entities::badge::{Badge, PublicBadge},
    error,
};

#[post("/api/badge")]
pub async fn post_badge(
    context: GeneralContext,
    Json(data): web::Json<CreateBadge>,
) -> error::Result<Json<Badge<String>>> {
    Ok(Json(BadgeService::new(context).create(data).await?))
}

#[get("/api/badge/{email}")]
pub async fn find_badge(
    context: GeneralContext,
    email: web::Path<String>,
) -> error::Result<Json<Option<PublicBadge>>> {
    Ok(Json(
        BadgeService::new(context)
            .find_by_email(email.into_inner())
            .await?,
    ))
}

#[patch("/api/badge/merge/{secret}")]
pub async fn merge(
    context: GeneralContext,
    secret: web::Path<String>,
) -> error::Result<HttpResponse> {
    BadgeService::new(context)
        .merge(secret.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[delete("/api/badge/delete/{secret}")]
pub async fn delete(
    context: GeneralContext,
    secret: web::Path<String>,
) -> error::Result<HttpResponse> {
    BadgeService::new(context)
        .delete(secret.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}
