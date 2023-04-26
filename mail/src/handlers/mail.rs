use actix_web::{
    post,
    web::{self},
    HttpResponse,
};
use common::{context::Context, entities::letter::CreateLetter, error};

use crate::service::mail::{CreateFeedback, MailService};

#[post("/api/mail")]
pub async fn send_mail(
    context: Context,
    letter: web::Json<CreateLetter>,
) -> error::Result<HttpResponse> {
    MailService::new(context)
        .send_letter(letter.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/api/feedback")]
pub async fn send_feedback(
    context: Context,
    letter: web::Json<CreateFeedback>,
) -> error::Result<HttpResponse> {
    MailService::new(context)
        .feedback(letter.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}