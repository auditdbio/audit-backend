use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, patch, web::{Path, Json, Query}, HttpResponse};
use std::collections::HashMap;

use common::{
    api::file::ChangeFile,
    context::GeneralContext,
    error,
};

use crate::service::file::{FileService, Metadata};

#[post("/file")]
pub async fn create_file(
    context: GeneralContext,
    payload: Multipart,
) -> error::Result<HttpResponse> {
    FileService::new(context)
        .create_file(payload)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/file/name/{filename:.*}")]
pub async fn find_file(
    context: GeneralContext,
    filename: Path<String>,
    query: Query<HashMap<String, String>>,
) -> error::Result<NamedFile> {
    let code = query.get("code");
    FileService::new(context)
        .find_file(filename.into_inner(), code)
        .await
}

#[get("/file/id/{id}")]
pub async fn get_file_by_id(
    context: GeneralContext,
    file_id: Path<String>,
) -> error::Result<NamedFile> {
    FileService::new(context)
        .get_file_by_id(file_id.parse()?)
        .await
}

#[get("/file/meta/{id}")]
pub async fn get_meta_by_id(
    context: GeneralContext,
    file_id: Path<String>,
) -> error::Result<Json<Metadata>> {
    Ok(Json(
        FileService::new(context)
            .get_meta_by_id(file_id.parse()?)
            .await?
    ))
}

#[delete("/file/name/{filename:.*}")]
pub async fn delete_file(
    context: GeneralContext,
    filename: Path<String>,
) -> error::Result<HttpResponse> {
    FileService::new(context)
        .delete_file(filename.into_inner())
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/file/id/{id}")]
pub async fn delete_file_by_id(
    context: GeneralContext,
    file_id: Path<String>,
) -> error::Result<HttpResponse> {
    FileService::new(context)
        .delete_file_by_id(file_id.parse()?)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[patch("/file/name/{filename:.*}")]
pub async fn change_file(
    context: GeneralContext,
    filename: Path<String>,
    Json(data): Json<ChangeFile>,
) -> error::Result<HttpResponse> {
    FileService::new(context)
        .change_file(filename.into_inner(), data)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
