use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use common::{
    context::GeneralContext,
    entities::auditor::{Auditor, PublicAuditor},
    error,
};

use serde_json::json;

use crate::service::auditor::{AuditorChange, AuditorService, CreateAuditor};

#[post("/auditor")]
pub async fn post_auditor(
    context: GeneralContext,
    Json(data): web::Json<CreateAuditor>,
) -> error::Result<Json<Auditor<String>>> {
    Ok(Json(AuditorService::new(context).create(data).await?))
}

#[get("/auditor/{id}")]
pub async fn get_auditor(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let service = AuditorService::new(context);
    let id = id.parse()?;
    if let Some(res) = service.find(id).await? {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/auditor_by_link_id/{link_id}")]
pub async fn find_by_link_id(
    context: GeneralContext,
    link_id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let res = AuditorService::new(context)
        .find_by_link_id(link_id.parse()?)
        .await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/my_auditor")]
pub async fn get_my_auditor(context: GeneralContext) -> error::Result<HttpResponse> {
    let res = AuditorService::new(context).my_auditor().await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[patch("/my_auditor")]
pub async fn patch_auditor(
    context: GeneralContext,
    Json(data): Json<AuditorChange>,
) -> error::Result<Json<Auditor<String>>> {
    Ok(Json(AuditorService::new(context).change(data).await?))
}

#[delete("/auditor/{id}")]
pub async fn delete_auditor(
    context: GeneralContext,
    id: web::Path<String>,
) -> error::Result<Json<PublicAuditor>> {
    Ok(Json(
        AuditorService::new(context).delete(id.parse()?).await?,
    ))
}
