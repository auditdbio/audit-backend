use actix_web::{
    delete, get, patch, post,
    web::{self, Json}, HttpResponse,
};

use common::{
    context::Context, error,
};

use serde_json::json;

use crate::service::auditor::{PublicAuditor, AuditorService, CreateAuditor, AuditorChange};


#[post("/api/auditor")]
pub async fn post_auditor(
    context: Context,
    Json(data): web::Json<CreateAuditor>,
) -> error::Result<Json<PublicAuditor>> {
    Ok(Json(AuditorService::new(context).create(data).await?))
}

#[get("/api/auditor/{id}")]
pub async fn get_auditor(
    context: Context,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let res = AuditorService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json!{{}}))
    }
}


#[patch("/api/auditor/{id}")]
pub async fn patch_auditor(
    context: Context,
    id: web::Path<String>,
    Json(data): Json<AuditorChange>,
) -> error::Result<Json<PublicAuditor>> {
    Ok(Json(AuditorService::new(context).change(id.parse()?, data).await?))
}

#[delete("/api/auditor/{id}")]
pub async fn delete_auditor(
    context: Context,
    id: web::Path<String>,
) -> error::Result<Json<PublicAuditor>> {
    Ok(Json(AuditorService::new(context).delete(id.parse()?).await?))
}
