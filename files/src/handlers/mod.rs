use actix_web::{
    get, post,
    web::{self, Bytes, Path},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use common::auth_session::{self, get_auth_session, AuthSession, SessionManager};
use mongodb::bson::oid::ObjectId;

use crate::repositories::{
    files::FilesRepository,
    meta::{MetaData, MetaRepository},
};

#[post("/api/files/create")]
pub async fn create_file(
    req: HttpRequest,
    file: Bytes,
    files_repo: web::Data<FilesRepository>,
    meta_repo: web::Data<MetaRepository>,
    manager: web::Data<auth_session::AuthSessionManager>,
) -> HttpResponse {
    let session = manager.get_session(req.clone().into()).await.unwrap();

    let id = files_repo.create(file).await;

    let meta_information = MetaData {
        id,
        creator_id: session.user_id(),
        last_modified: Utc::now().naive_utc(),
        content_type: req
            .headers()
            .get("Content-Type")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    };
    meta_repo.create(meta_information).await.unwrap();

    HttpResponse::Ok().json(id)
}

#[get("/api/files/create/{id}")]
pub async fn get_file(
    req: HttpRequest,
    id: Path<ObjectId>,
    files_repo: web::Data<FilesRepository>,
    meta_repo: web::Data<MetaRepository>,
    manager: web::Data<auth_session::AuthSessionManager>,
) -> HttpResponse {
    let session = manager.get_session(req.clone().into()).await.unwrap();
    let id = id.into_inner();

    let file = files_repo.get(&id).await;
    let meta_information = meta_repo.find(&id).await.unwrap().unwrap();

    HttpResponse::Ok()
        .append_header(("Content-Type", meta_information.content_type))
        .body(file)
}
