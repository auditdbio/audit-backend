use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use common::{context::Context, error, entities::{audit_request::AuditRequest, role::Role}};

use serde_json::json;

use crate::service::audit_request::{CreateRequest, PublicRequest, RequestChange, RequestService};

#[post("/api/audit_request")]
pub async fn post_audit_request(
    context: Context,
    Json(data): web::Json<CreateRequest>,
) -> error::Result<Json<PublicRequest>> {
    Ok(Json(RequestService::new(context).create(data).await?))
}

#[get("/api/audit_request/{id}")]
pub async fn get_audit_request(
    context: Context,
    id: web::Path<String>,
) -> error::Result<HttpResponse> {
    let res = RequestService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/my_audit_request/{role}")]
pub async fn get_my_audit_request(context: Context, role: web::Path<Role>) -> error::Result<Json<Vec<AuditRequest<String>>>> {
    Ok(Json(RequestService::new(context).my_request(role.into_inner()).await?))
}

#[patch("/api/audit_request/{id}")]
pub async fn patch_audit_request(
    context: Context,
    id: web::Path<String>,
    Json(data): Json<RequestChange>,
) -> error::Result<Json<PublicRequest>> {
    Ok(Json(
        RequestService::new(context)
            .change(id.parse()?, data)
            .await?,
    ))
}

#[delete("/api/audit_request/{id}")]
pub async fn delete_audit_request(
    context: Context,
    id: web::Path<String>,
) -> error::Result<Json<PublicRequest>> {
    Ok(Json(
        RequestService::new(context).delete(id.parse()?).await?,
    ))
}
