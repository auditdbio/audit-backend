use crate::{
    auth::Auth,
    context::Context,
    error,
    services::{MAIL_SERVICE, PROTOCOL},
};

pub async fn post_code(context: &Context, auth: Auth, payload: String) -> error::Result<String> {
    Ok(context
        .make_request::<String>()
        .auth(&auth)
        .post(format!(
            "{}://{}/api/code/{}",
            PROTOCOL.as_str(),
            MAIL_SERVICE.as_str(),
            payload
        ))
        .send()
        .await?
        .json::<String>()
        .await?)
}

pub async fn get_code(
    context: &Context,
    auth: Auth,
    code: String,
) -> error::Result<Option<String>> {
    Ok(context
        .make_request::<String>()
        .auth(&auth)
        .get(format!(
            "{}://{}/api/code/{}",
            PROTOCOL.as_str(),
            MAIL_SERVICE.as_str(),
            code
        ))
        .send()
        .await?
        .json::<Option<String>>()
        .await?)
}
