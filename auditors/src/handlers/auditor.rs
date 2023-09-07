use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use common::{
    context::Context,
    entities::auditor::{Auditor, PublicAuditor},
    error,
};

use serde_json::json;

use crate::service::auditor::{AuditorChange, AuditorService, CreateAuditor};

#[post("/api/auditor")]
pub async fn post_auditor(
    context: Context,
    Json(data): web::Json<CreateAuditor>,
) -> error::Result<Json<Auditor<String>>> {
    Ok(Json(AuditorService::new(context).create(data).await?))
}

#[get("/api/auditor/{id}")]
pub async fn get_auditor(context: Context, id: web::Path<String>) -> error::Result<HttpResponse> {
    let service = AuditorService::new(context);
    let id = id.parse()?;
    if let Some(res) = service.find(id).await? {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/my_auditor")]
pub async fn get_my_auditor(context: Context) -> error::Result<HttpResponse> {
    let res = AuditorService::new(context).my_auditor().await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[patch("/api/my_auditor")]
pub async fn patch_auditor(
    context: Context,
    Json(data): Json<AuditorChange>,
) -> error::Result<Json<Auditor<String>>> {
    Ok(Json(AuditorService::new(context).change(data).await?))
}

#[delete("/api/auditor/{id}")]
pub async fn delete_auditor(
    context: Context,
    id: web::Path<String>,
) -> error::Result<Json<PublicAuditor>> {
    Ok(Json(
        AuditorService::new(context).delete(id.parse()?).await?,
    ))
}
