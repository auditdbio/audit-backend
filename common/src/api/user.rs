use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::user::PublicUser,
    error,
    services::{PROTOCOL, USERS_SERVICE},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: String,
    pub current_role: String,
    pub use_email: Option<bool>,
    pub admin_creation_password: Option<String>,
    pub secret: Option<String>,
}

pub async fn get_by_id(
    context: &GeneralContext,
    auth: Auth,
    id: ObjectId,
) -> error::Result<PublicUser> {
    Ok(context
        .make_request::<PublicUser>()
        .get(format!(
            "{}://{}/api/user/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            id
        ))
        .auth(auth)
        .send()
        .await?
        .json::<PublicUser>()
        .await?)
}

pub async fn get_by_email(
    context: &GeneralContext,
    email: String,
) -> error::Result<Option<PublicUser>> {
    Ok(context
        .make_request::<PublicUser>()
        .get(format!(
            "{}://{}/api/user_by_email/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            email
        ))
        .auth(context.server_auth())
        .send()
        .await?
        .json::<PublicUser>()
        .await
        .ok())
}
