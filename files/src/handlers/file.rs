use actix_multipart::Multipart;
use actix_web::{post, HttpResponse};
use common::{context::Context, error};
use futures::StreamExt;

use crate::service::file::FileService;

#[post("/api/file")]
pub async fn create_file(context: Context, mut payload: Multipart) -> error::Result<HttpResponse> {
    let mut file = vec![];
    let mut path = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| anyhow::anyhow!("{}", e))?;

        match field.name() {
            "file" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|e| anyhow::anyhow!("{}", e))?;
                    file.push(data);
                }
            }
            "path" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|e| anyhow::anyhow!("{}", e))?;
                    path.push_str(&String::from_utf8(data.to_vec()).unwrap());
                }
            }
            _ => (),
        }
    }
    
    FileService::new(context).create_file(path, file.concat()).await?;

    Ok(HttpResponse::Ok().finish())
}


pub async fn find_file() {}

pub async fn find_file_with_cookie() {}


pub async fn change_file() {}

pub async fn delete_file() {}
