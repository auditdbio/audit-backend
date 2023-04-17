use std::{path::Path, str::FromStr};

use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use common::{
    auth_session::{self, get_auth_header, SessionManager},
    entities::audit::Audit,
    services::AUDITS_SERVICE,
};
use mongodb::bson::oid::ObjectId;

use actix_web::Error;
use futures_util::StreamExt as _;
use reqwest::Client;
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

fn get_extension_from_filename(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string()
}

pub(super) async fn get_audit(
    client: &Client,
    id: &ObjectId,
    auth: String,
) -> Option<Audit<String>> {
    let res = client
        .get(format!(
            "https://{}/api/audits/by_id/{}",
            AUDITS_SERVICE.as_str(),
            id.to_hex()
        ))
        .header("Authorization", auth)
        .send()
        .await
        .unwrap();
    match res.json::<Audit<String>>().await {
        Ok(body) => Some(body),
        Err(e) => {
            log::error!("Failed to get audit: {}", e);
            None
        }
    }
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
    let _session = manager
        .get_session(req.clone().into())
        .await
        .unwrap()
        .unwrap();
    let mut file = vec![];
    let mut path = String::new();
    let mut private = false;
    let mut original_name = String::new();
    let mut auditor_id = String::new();
    let mut customer_id = String::new();

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
            "original_name" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    original_name.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            "private" => {
                let mut str = String::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }

                private = str == "true";
            }
            "auditorId" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    auditor_id.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            "customerId" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    customer_id.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            _ => (),
        }
    }
    let allowed_users = if private {

        vec![
            ObjectId::from_str(&auditor_id).unwrap(),
            ObjectId::from_str(&customer_id).unwrap(),
        ]
    } else {
        Vec::new()
    };

    let extension = get_extension_from_filename(&original_name);

    let full_path = format!("{}.{}", path, extension);

    meta_repo.delete(path.clone()).await.unwrap();

    files_repo.create(file.concat(), full_path).await;

    let meta_information = Metadata {
        id: ObjectId::new(),
        private,
        extension: get_extension_from_filename(&original_name),
        allowed_users,
        last_modified: Utc::now().timestamp_micros(),
        path,
    };
    meta_repo.create(&meta_information).await.unwrap();

    Ok(HttpResponse::Ok().into())
}

#[get("/api/files/get/{filename:.*}")]
pub async fn get_file(
    req: HttpRequest,
    filename: web::Path<String>,
    _files_repo: web::Data<FilesRepository>,
    meta_repo: web::Data<MetadataRepo>,
    manager: web::Data<auth_session::AuthSessionManager>,
) -> HttpResponse {
    let session = manager.get_session(req.clone().into()).await.unwrap(); // TODO: remove unwrap

    let path: std::path::PathBuf = filename.parse().unwrap();
    let file_path = format!("/auditdb-files/{}", filename);

    let metadata = meta_repo
        .find_by_path(path.to_str().unwrap().to_string())
        .await
        .unwrap()
        .unwrap();

    if let Some(auth_session) = session {
        if !metadata.allowed_users.contains(&auth_session.user_id) && metadata.private {
            return HttpResponse::BadRequest().body("You are not allowed to access this file");
        }
    } else if metadata.private {
        return HttpResponse::BadRequest().body("You are not allowed to access this file");
    }

    let full_path = format!("{}.{}", file_path, metadata.extension);

    let file = actix_files::NamedFile::open_async(full_path).await.unwrap();
    file.into_response(&req)
}
