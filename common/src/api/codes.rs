use crate::{
    context::GeneralContext,
    error,
    services::{API_PREFIX, MAIL_SERVICE, PROTOCOL},
};

pub async fn post_code(context: &GeneralContext, payload: String) -> error::Result<String> {
    Ok(context
        .make_request::<String>()
        .post(format!(
            "{}://{}/{}/code/{}",
            PROTOCOL.as_str(),
            MAIL_SERVICE.as_str(),
            API_PREFIX.as_str(),
            payload
        ))
        .send()
        .await?
        .json::<String>()
        .await?)
}

pub async fn get_code(context: &GeneralContext, code: String) -> error::Result<Option<String>> {
    Ok(context
        .make_request::<()>()
        .get(format!(
            "{}://{}/{}/code/{}",
            PROTOCOL.as_str(),
            MAIL_SERVICE.as_str(),
            API_PREFIX.as_str(),
            code
        ))
        .send()
        .await?
        .json::<Option<String>>()
        .await?)
}
