use actix_web::{
    post,
    web::{self, Json},
};
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
    pub error: Option<String>,
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

    if let Ok(email) = email.clone().parse() {
        let Ok(email) = Message::builder()
            .from(EMAIL_ADDRESS.to_string().parse().unwrap())
            .to(email)
            .subject("Welcome to AuditDB waiting list!")
            .body(include_str!("../../templates/waiting-letter.txt").to_string()) else {
                return Ok(web::Json(PostElementResponse {
                    id: elem.id.to_hex(),
                    email: elem.email,
                    error: Some("Error creating message builder".to_string()),
                }));
            };
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(
                EMAIL_ADDRESS.to_string(),
                EMAIL_PASSWORD.to_string(),
            ))
            .build();
        if let Err(err) = mailer.send(&email) {
            return Ok(web::Json(PostElementResponse {
                id: elem.id.to_hex(),
                email: elem.email,
                error: Some(format!("Error sending email: {}", err)),
            }));
        }
    }

    repo.create(&elem).await?;
    return Ok(web::Json(PostElementResponse {
        id: elem.id.to_hex(),
        email: elem.email,
        error: None,
    }));
}
