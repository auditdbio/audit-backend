use crate::{
    context::GeneralContext,
    entities::letter::CreateLetter,
    error,
    services::{API_PREFIX, MAIL_SERVICE, PROTOCOL},
};

pub async fn send_mail(context: &GeneralContext, create_letter: CreateLetter) -> error::Result<()> {
    context
        .make_request::<CreateLetter>()
        .auth(context.server_auth())
        .post(format!(
            "{}://{}/{}/mail",
            PROTOCOL.as_str(),
            MAIL_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .json(&create_letter)
        .send()
        .await?;
    Ok(())
}
