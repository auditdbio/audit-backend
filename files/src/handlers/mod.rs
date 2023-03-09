use actix_multipart::Multipart;
use actix_web::{
    post,
    web::{self, Bytes},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use common::auth_session::{self, SessionManager};
use mongodb::bson::oid::ObjectId;

use actix_web::{Error};
use futures_util::StreamExt as _;

use crate::repositories::{
    files::FilesRepository,
    meta::{Metadata, MetadataRepo},
};

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = FilePath
    ),
    responses(
        (status = 200)
    )
)]
#[post("/api/files/create")]
async fn create_file(req: HttpRequest, mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Some(item) = payload.next().await {
           let mut field = item?;

           // Field in turn is stream of *Bytes* object
           while let Some(chunk) = field.next().await {
               log::info!("-- CHUNK: \n{:?}", std::str::from_utf8(&chunk?));
           }
    }

    Ok(HttpResponse::Ok().into())
}

// #[utoipa::path(
//     params(
//         ("Authorization" = String, Header,  description = "Bearer token"),
//     ),
//     request_body(
//         content = FilePath
//     ),
//     responses(
//         (status = 200)
//     )
// )]
// #[post("/api/files/create")]
// pub async fn create_file(
//     req: HttpRequest,
//     file: Bytes,
//     files_repo: web::Data<FilesRepository>,
//     meta_repo: web::Data<MetadataRepo>,
//     manager: web::Data<auth_session::AuthSessionManager>,
// ) -> HttpResponse {
//     let session = manager.get_session(req.clone().into()).await.unwrap();

//     files_repo.create(file, path.path.clone()).await;

//     let meta_information = Metadata {
//         id: ObjectId::new(),
//         creator_id: session.user_id(),
//         last_modified: Utc::now().naive_utc(),
//         content_type: req
//             .headers()
//             .get("Content-Type")
//             .unwrap()
//             .to_str()
//             .unwrap()
//             .to_string(),
//         path: path.path.clone(),
//     };
//     meta_repo.create(&meta_information).await.unwrap();
//     HttpResponse::Ok()
//     .streaming(stream)
// }

// #[utoipa::path(
//     params(
//         ("Authorization" = String, Header,  description = "Bearer token"),
//     ),
//     request_body(
//         content = FilePath
//     ),
//     responses(
//         (status = 200)
//     )
// )]
// #[post("/api/files/get")]
// pub async fn get_file(
//     req: HttpRequest,
//     path: web::Json<FilePath>,
//     files_repo: web::Data<FilesRepository>,
//     meta_repo: web::Data<MetadataRepo>,
//     manager: web::Data<auth_session::AuthSessionManager>,
// ) -> HttpResponse {
//     let _session = manager.get_session(req.clone().into()).await.unwrap();

//     let file = files_repo.get(path.path.clone()).await;
//     let meta_information = meta_repo
//         .find_by_path(path.path.clone())
//         .await
//         .unwrap()
//         .unwrap();

//     HttpResponse::Ok()
//         .append_header(("Content-Type", meta_information.content_type))
//         .body(file)
// }

// pub async fn patch_file() {

// }

// pub async fn delete_file() {}
