use mongodb::bson::oid::ObjectId;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::user::{LinkedAccount, PublicUser, User},
    error,
    services::{API_PREFIX, PROTOCOL, USERS_SERVICE},
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
    pub linked_accounts: Option<Vec<LinkedAccount>>,
    pub is_passwordless: Option<bool>,
}

pub async fn get_by_id(
    context: &GeneralContext,
    auth: Auth,
    id: ObjectId,
) -> error::Result<PublicUser> {
    Ok(context
        .make_request::<PublicUser>()
        .get(format!(
            "{}://{}/{}/user/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            id
        ))
        .auth(auth)
        .send()
        .await?
        .json::<PublicUser>()
        .await?)
}

pub async fn get_by_link_id(
    context: &GeneralContext,
    auth: Auth,
    link_id: String,
) -> error::Result<PublicUser> {
    Ok(context
        .make_request::<PublicUser>()
        .get(format!(
            "{}://{}/{}/user_by_link_id/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            link_id,
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
) -> error::Result<Option<User<ObjectId>>> {
    Ok(context
        .make_request::<User<ObjectId>>()
        .get(format!(
            "{}://{}/{}/user_by_email/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            email
        ))
        .auth(context.server_auth())
        .send()
        .await?
        .json::<User<ObjectId>>()
        .await
        .ok())
}

pub fn validate_name(name: &str) -> bool {
    let regex = Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
    regex.is_match(name)
}
