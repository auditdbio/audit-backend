use actix_web::{
    post,
    web::{self},
    HttpResponse,
};
use common::{context::Context, error};

use crate::service::mail::{MailService, CreateLetter};

#[post("/api/mail")]
pub async fn sent_mail(context: Context, letter: web::Json<CreateLetter>) -> error::Result<HttpResponse> {
    MailService::new(context)
        .send_mail(letter.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}
