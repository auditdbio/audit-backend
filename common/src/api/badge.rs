use std::process::id;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    context::Context,
    entities::badge::PublicBadge,
    error,
    services::{AUDITORS_SERVICE, PROTOCOL},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BadgePayload {
    pub badge_id: ObjectId,
    pub email: String,
}

pub async fn merge(context: &Context, auth: Auth, secret: String) -> error::Result<()> {
    context
        .make_request::<()>()
        .get(format!(
            "{}://{}/api/badge/merge/{}",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            secret
        ))
        .auth(auth)
        .send()
        .await?;

    Ok(())
}

pub async fn get_badge(context: &Context, email: String) -> error::Result<Option<PublicBadge>> {
    Ok(context
        .make_request::<()>()
        .get(format!(
            "{}://{}/api/badge/{}",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            email
        ))
        .send()
        .await?
        .json::<Option<PublicBadge>>()
        .await?)
}
