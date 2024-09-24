use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web::{Path, Json}, HttpResponse};
use common::{context::GeneralContext, error};

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
) -> error::Result<NamedFile> {
    FileService::new(context)
        .find_file(filename.into_inner())
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

#[delete("/file/{filename:.*}")]
pub async fn delete_file(
    context: GeneralContext,
    filename: Path<String>,
) -> error::Result<HttpResponse> {
    FileService::new(context)
        .delete_file(filename.into_inner())
        .await?;

    Ok(HttpResponse::Ok().finish())
}
