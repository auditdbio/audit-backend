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
    pub linked_accounts: Option<Vec<LinkedAccounts>>,
    pub is_passwordless: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkedAccounts {
    pub id: i32,
    pub name: String,
    pub email: String,
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
  pub name: String,
  pub avatar_url: String,
  pub company: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubUserEmails {
  pub email: String,
  pub primary: bool,
  pub verified: bool,
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
