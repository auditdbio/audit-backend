use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
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

pub async fn get_by_id(context: &GeneralContext, id: ObjectId) -> error::Result<PublicUser> {
    Ok(context
        .make_request::<PublicUser>()
        .get(format!(
            "{}://{}/api/user/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            id
        ))
        .send()
        .await?
        .json::<PublicUser>()
        .await?)
}
