use mongodb::bson::oid::ObjectId;

use crate::{
    context::GeneralContext,
    error,
    entities::{
        organization::PublicOrganization,
    },
    services::{API_PREFIX, USERS_SERVICE, PROTOCOL},
};

pub async fn get_organization(context: &GeneralContext, id: ObjectId) -> error::Result<PublicOrganization> {
    Ok(context
        .make_request::<PublicOrganization>()
        .auth(context.auth())
        .get(format!(
            "{}://{}/{}/organization/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            id,
        ))
        .send()
        .await?
        .json::<PublicOrganization>()
        .await?)
}