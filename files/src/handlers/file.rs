use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{get, post, web::Path, HttpResponse, delete};
use common::{context::Context, error};
use futures::StreamExt;

use crate::service::file::FileService;

#[post("/api/file")]
pub async fn create_file(context: Context, mut payload: Multipart) -> error::Result<HttpResponse> {
    let mut file = vec![];
    let mut path = String::new();

    let mut private = false;
    let mut original_name = String::new();
    let mut audit_id = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        match field.name() {
            "file" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    file.push(data);
                }
            }
            "path" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    path.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            "original_name" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    original_name.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            "private" => {
                let mut str = String::new();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    str.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }

                private = str == "true";
            }
            "audit" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    audit_id.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            _ => (),
        }
    }

    let allowed_users = Vec::new();

    FileService::new(context)
        .create_file(path, allowed_users, private, original_name, file.concat())
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/api/file/{filename:.*}")]
pub async fn find_file(context: Context, filename: Path<String>) -> error::Result<NamedFile> {
    Ok(FileService::new(context)
        .find_file(filename.into_inner())
        .await?)
}

#[delete("/api/file/{filename:.*}")]
pub async fn delete_file(context: Context, filename: Path<String>) -> error::Result<NamedFile> {
    Ok(FileService::new(context)
        .find_file(filename.into_inner())
        .await?)
}
