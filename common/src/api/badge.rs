use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::badge::PublicBadge,
    error,
    services::{API_PREFIX, AUDITORS_SERVICE, PROTOCOL},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BadgePayload {
    pub badge_id: ObjectId,
    pub email: String,
}

pub async fn merge(context: &GeneralContext, auth: Auth, secret: String) -> error::Result<()> {
    context
        .make_request::<()>()
        .patch(format!(
            "{}://{}/{}/badge/merge/{}",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            secret
        ))
        .auth(auth)
        .send()
        .await?;

    Ok(())
}

pub async fn get_badge(
    context: &GeneralContext,
    email: String,
) -> error::Result<Option<PublicBadge>> {
    Ok(context
        .make_request::<()>()
        .get(format!(
            "{}://{}/{}/badge/{}",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            email
        ))
        .send()
        .await?
        .json::<Option<PublicBadge>>()
        .await?)
}
