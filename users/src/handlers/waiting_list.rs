use actix_web::{
    post,
    web::{self, Json},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use check_if_email_exists::{check_email, CheckEmailInput, CheckEmailInputProxy, Reachable};


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

    let mut input = CheckEmailInput::new(email.clone());

    let result = check_email(&input).await;

    if result.is_reachable == Reachable::Safe || result.is_reachable == Reachable::Safe {
        let email = Message::builder()
            .from(EMAIL_ADDRESS.clone().parse().unwrap())
            .to(email.clone().parse().unwrap())
            .subject("Welcome to AuditDB waiting list!")
            .body(include_str!("../../templates/waiting-letter.txt").to_string())
            .unwrap();
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(EMAIL_ADDRESS.clone(), EMAIL_PASSWORD.clone()))
            .build();
        mailer.send(&email).unwrap();
    }

    

    repo.create(&elem).await?;
    return Ok(web::Json(PostElementResponse {
        id: elem.id.to_hex(),
        email: elem.email,
    }));
}
