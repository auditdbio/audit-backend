use std::io::Read;

use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use common::auth_session::{self, SessionManager};
use mongodb::bson::oid::ObjectId;

use actix_web::Error;
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::repositories::{
    files::FilesRepository,
    meta::{Metadata, MetadataRepo},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FilePath {
    pub path: String,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200)
    )
)]
#[post("/api/files/create")]
async fn create_file(
    req: HttpRequest,
    mut payload: Multipart,
    files_repo: web::Data<FilesRepository>,
    meta_repo: web::Data<MetadataRepo>,
    manager: web::Data<auth_session::AuthSessionManager>,
) -> Result<HttpResponse, Error> {
    let session = manager.get_session(req.clone().into()).await.unwrap();
    let mut file = vec![];
    let mut path = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;

        match field.name() {
            "file" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    file.push(data);
                }
            }
            "path" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    path.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            _ => (),
        }
    }
    files_repo.create(file.concat(), path.clone()).await;

    let meta_information = Metadata {
        id: ObjectId::new(),
        creator_id: session.user_id(),
        last_modified: Utc::now().timestamp_micros(),
        path,
    };
    meta_repo.create(&meta_information).await.unwrap();

    Ok(HttpResponse::Ok().into())
}

#[get("/api/files/get/{filename:.*}")]
pub async fn get_file(
    req: HttpRequest,
    files_repo: web::Data<FilesRepository>,
    meta_repo: web::Data<MetadataRepo>,
    manager: web::Data<auth_session::AuthSessionManager>,
) -> HttpResponse {
    let auth = req
        .cookie("Authorization")
        .unwrap()
        .to_string()
        .strip_prefix("Authorization=")
        .unwrap()
        .to_string();
    let session = manager.get_session_from_string(auth).await.unwrap();

    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    let file_path = format!("/auditdb-files/{}", path.to_str().unwrap());

    let metadata = meta_repo
        .find_by_path(path.to_str().unwrap().to_string())
        .await
        .unwrap()
        .unwrap();

    if metadata.creator_id != session.user_id() {
        return HttpResponse::BadRequest().body("You are not allowed to access this file");
    }

    let file = actix_files::NamedFile::open_async(file_path).await.unwrap();
    log::info!("{:?}", file.try_clone().unwrap().bytes());
    file.into_response(&req)
}
