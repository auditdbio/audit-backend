use crate::{
    context::Context,
    entities::letter::CreateLetter,
    error,
    services::{MAIL_SERVICE, PROTOCOL},
};

pub async fn send_mail(context: &Context, create_letter: CreateLetter) -> error::Result<()> {
    context
        .make_request::<CreateLetter>()
        .auth(context.server_auth())
        .post(format!(
            "{}://{}/api/mail",
            PROTOCOL.as_str(),
            MAIL_SERVICE.as_str(),
        ))
        .json(&create_letter)
        .send()
        .await?;
    Ok(())
}
