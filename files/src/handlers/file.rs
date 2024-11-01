use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{
    web::{Path, Json, Query},
    delete, get, post, patch,
    HttpResponse, HttpRequest,
};
use std::collections::HashMap;
use mongodb::bson::oid::ObjectId;

use common::{
    api::file::{ChangeFile, PublicMetadata},
    context::GeneralContext,
    error,
};

use crate::service::file::FileService;

#[post("/file")]
pub async fn create_file(
    context: GeneralContext,
    payload: Multipart,
) -> error::Result<Json<PublicMetadata>> {
    Ok(Json(
        FileService::new(context)
            .create_file(payload)
            .await?
    ))
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
    query: Query<HashMap<String, String>>,
) -> error::Result<NamedFile> {
    let code = query.get("code");
    let parsed_id = file_id.parse::<ObjectId>();

    match parsed_id {
        Ok(file_id) => {
            FileService::new(context)
                .get_file_by_id(file_id, code)
                .await
        }
        Err(_) => {
            FileService::new(context)
                .find_file(file_id.into_inner(), code)
                .await
        }
    }
}

#[get("/file/meta/{id}")]
pub async fn get_meta_by_id(
    context: GeneralContext,
    file_id: Path<String>,
    query: Query<HashMap<String, String>>,
) -> error::Result<Json<PublicMetadata>> {
    let code = query.get("code");
    Ok(Json(
        FileService::new(context)
            .get_meta_by_id(file_id.parse()?, code)
            .await?
    ))
}

#[delete("/file/name/{filename:.*}")]
pub async fn delete_file(
    context: GeneralContext,
    filename: Path<String>,
) -> error::Result<HttpResponse> {
    FileService::new(context)
        .delete_file_by_name(filename.into_inner())
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

#[post("/file/get_and_delete/{id}")]
pub async fn get_and_delete_by_id(
    context: GeneralContext,
    file_id: Path<String>,
    req: HttpRequest,
) -> error::Result<HttpResponse> {
    let file = FileService::new(context)
        .get_and_delete_by_id(file_id.parse()?)
        .await?;

    Ok(file.into_response(&req))
}

#[patch("/file/name/{filename:.*}")]
pub async fn change_file_meta(
    context: GeneralContext,
    filename: Path<String>,
    Json(data): Json<ChangeFile>,
) -> error::Result<HttpResponse> {
    FileService::new(context)
        .change_file_meta_by_name(filename.into_inner(), data)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[patch("/file/id/{id}")]
pub async fn change_file_meta_by_id(
    context: GeneralContext,
    file_id: Path<String>,
    Json(data): Json<ChangeFile>,
) -> error::Result<HttpResponse> {
    FileService::new(context)
        .change_file_meta_by_id(file_id.parse()?, data)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
