use mongodb::bson::oid::ObjectId;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubAuth {
    pub code: String,
    pub current_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetGithubAccessToken {
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubAccessResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubUserData {
    pub id: i32,
    pub login: String,
    pub name: Option<String>,
    pub html_url: String,
    pub avatar_url: String,
    pub company: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubUserEmails {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
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
