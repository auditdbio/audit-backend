use actix_web::{
    post,
    web::{self, Json},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Result;
use crate::repositories::list_element::{ListElement, ListElementRepository};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostElement {
    pub email: String,
}

#[utoipa::path(
    request_body(
        content = PostElement
    ),
    responses(
        (status = 200, body = ListElement)
    )
)]
#[post("/api/waiting_list")]
pub async fn post_element(
    Json(data): web::Json<PostElement>,
    repo: web::Data<ListElementRepository>,
) -> Result<web::Json<ListElement>> {
    let elem = ListElement {
        id: ObjectId::new(),
        email: data.email,
    };

    repo.create(&elem).await?;
    return Ok(web::Json(elem));
}
