use actix_web::{
    post,
    web::{self, Json},
};
use log::info;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::error::Result;
use crate::repositories::list_element::{ListElement, ListElementRepository};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostElementRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostElementResponse {
    pub id: String,
    pub email: String,
}

lazy_static::lazy_static! {
    static ref EMAIL_ADDRESS: String = std::env::var("HELLO_MAIL_ADDRESS").unwrap();
    static ref EMAIL_PASSWORD: String = std::env::var("HELLO_MAIL_PASSWORD").unwrap();
}

#[utoipa::path(
    request_body(
        content = PostElementRequest
    ),
    responses(
        (status = 200, body = PostElementResponse)
    )
)]
#[post("/api/waiting_list")]
pub async fn post_element(
    Json(data): web::Json<PostElementRequest>,
    repo: web::Data<ListElementRepository>,
) -> Result<web::Json<PostElementResponse>> {
    let email = data.email;

    let elem = ListElement {
        id: ObjectId::new(),
        email: email.clone(),
    };

    if let Ok(email) = Message::builder()
        .from(EMAIL_ADDRESS.clone().parse().unwrap())
        .to(email.clone().parse().unwrap())
        .subject("Welcome to AuditDB waiting list!")
        .body(include_str!("../../templates/waiting-letter.txt").to_string()) {
            let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(
                EMAIL_ADDRESS.clone(),
                EMAIL_PASSWORD.clone(),
            ))
            .build();
        if let Err(err) = mailer.send(&email) {
            info!("Error sending email: {:?}", err);
        }
    }
    

    repo.create(&elem).await?;
    return Ok(web::Json(PostElementResponse {
        id: elem.id.to_hex(),
        email: elem.email,
    }));
}
