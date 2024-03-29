use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, web::Path, HttpResponse};
use common::{context::GeneralContext, error};
use futures::StreamExt;
use mongodb::bson::oid::ObjectId;

use crate::service::file::FileService;

#[post("/file")]
pub async fn create_file(
    context: GeneralContext,
    mut payload: Multipart,
) -> error::Result<HttpResponse> {
    let mut file = vec![];
    let mut path = String::new();

    let mut private = false;
    let mut original_name = String::new();
    let mut customer_id = String::new();
    let mut auditor_id = String::new();
    let mut full_access = String::new();

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
            "customerId" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    customer_id.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            "auditorId" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    auditor_id.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }

            "full_access" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    full_access.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            _ => (),
        }
    }

    let mut full_access = full_access
        .split(' ')
        .filter_map(|id| id.trim().parse().ok())
        .collect::<Vec<ObjectId>>();

    if private {
        if let Ok(customer_id) = customer_id.parse() {
            full_access.push(customer_id);
        }

        if let Ok(auditor_id) = auditor_id.parse() {
            full_access.push(auditor_id);
        }
    }

    FileService::new(context)
        .create_file(path, full_access, private, original_name, file.concat())
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/file/{filename:.*}")]
pub async fn find_file(
    context: GeneralContext,
    filename: Path<String>,
) -> error::Result<NamedFile> {
    FileService::new(context)
        .find_file(filename.into_inner())
        .await
}

#[delete("/file/{filename:.*}")]
pub async fn delete_file(
    context: GeneralContext,
    filename: Path<String>,
) -> error::Result<NamedFile> {
    FileService::new(context)
        .find_file(filename.into_inner())
        .await
}
