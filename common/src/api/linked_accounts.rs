use serde::{Deserialize, Serialize};
use reqwest::{header, Client};
use std::env::var;
use crypto::{ aes, blockmodes, buffer::{self, ReadBuffer, WriteBuffer, BufferResult}};

use crate::{
    entities::user::LinkedAccount,
    error::{self, AddCode},
    services::{PROTOCOL, FRONTEND},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LinkedService {
    GitHub,
    X,
    LinkedIn,
    WalletConnect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddLinkedAccount {
    pub code: String,
    pub current_role: Option<String>,
    pub service: LinkedService,
    pub update_token: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateLinkedAccount {
    pub is_public: Option<bool>,
    pub token: Option<Vec<u8>>,
    pub scope: Option<String>,
    pub update_token: Option<bool>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetXAccessToken {
    pub code: String,
    pub client_id: String,
    pub grant_type: String,
    pub redirect_uri: String,
    pub code_verifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XAccessResponse {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserResponse {
    pub data: XUserData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XUserData {
    pub id: String,
    pub name: String,
    pub username: String,
    pub profile_image_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLinkedInAccessToken {
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkedInAccessResponse {
    pub access_token: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedInUserResponse {
    pub sub: String,
    pub name: String,
    pub given_name: String,
    pub email: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddWallet {
    pub address: String,
    pub message: String,
    pub signature: String,
}

pub async fn create_github_account(data: GetGithubAccessToken) -> error::Result<LinkedAccount> {
    let client = Client::new();

    let access_response = client
        .post(format!(
            "https://github.com/login/oauth/access_token?code={}&client_id={}&client_secret={}",
            data.code, data.client_id, data.client_secret,
        ))
        .header(header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    let access_json: GithubAccessResponse = serde_json::from_str(&access_response)?;
    let access_token = access_json.access_token;

    let user_response = client
        .get("https://api.github.com/user")
        .header(header::ACCEPT, "application/json")
        .header("User-Agent", "auditdbio")
        .bearer_auth(access_token.clone())
        .send()
        .await?
        .text()
        .await?;

    let emails_response = client
        .get("https://api.github.com/user/emails")
        .header(header::ACCEPT, "application/json")
        .header("User-Agent", "auditdbio")
        .bearer_auth(access_token.clone())
        .send()
        .await?
        .text()
        .await?;

    let user_data: GithubUserData = serde_json::from_str(&user_response)?;
    let emails: Vec<GithubUserEmails> = serde_json::from_str(&emails_response)?;

    let Some(email) = emails
        .iter()
        .find(|email| email.primary)
        .map(|email| email.email.to_string())
        else {
            return Err(anyhow::anyhow!("No email found").code(404));
        };

    let key = var("GITHUB_TOKEN_CRYPTO_KEY").unwrap();
    let iv = var("GITHUB_TOKEN_CRYPTO_IV").unwrap();

    let mut encryptor = aes::cbc_encryptor(
        aes::KeySize::KeySize256,
        key.as_bytes(),
        iv.as_bytes(),
        blockmodes::PkcsPadding,
    );

    let mut encrypted_data = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(access_token.as_bytes());
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = match encryptor.encrypt(
            &mut read_buffer,
            &mut write_buffer,
            true
        ) {
            Ok(value) => value,
            _ => return Err(anyhow::anyhow!("Encryption error").code(500))
        };

        encrypted_data.extend(write_buffer
            .take_read_buffer()
            .take_remaining()
            .iter()
            .map(|&i| i)
        );

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    let linked_account = LinkedAccount {
        id: user_data.id.to_string(),
        name: LinkedService::GitHub,
        email: email.clone(),
        url: user_data.html_url,
        avatar: user_data.avatar_url,
        is_public: false,
        username: user_data.login.clone(),
        token: Some(encrypted_data),
        scope: Some(access_json.scope),
    };

    Ok(linked_account)
}

pub async fn create_x_account(data: AddLinkedAccount) -> error::Result<LinkedAccount> {
    let client = Client::new();
    let client_id = var("X_CLIENT_ID").unwrap();
    let client_secret = var("X_CLIENT_SECRET").unwrap();

    let x_auth = GetXAccessToken {
        code: data.code,
        client_id: client_id.clone(),
        grant_type: "authorization_code".to_string(),
        redirect_uri: format!("{}://{}/oauth/callback", PROTOCOL.as_str(), FRONTEND.as_str()),
        code_verifier: "challenge".to_string(),
    };

    let access_response = client
        .post("https://api.twitter.com/2/oauth2/token")
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .basic_auth(client_id, Some(client_secret))
        .json(&x_auth)
        .send()
        .await?
        .text()
        .await?;

    let access_json: XAccessResponse = serde_json::from_str(&access_response)?;
    let access_token = access_json.access_token;

    let user_response = client
        .get("https://api.twitter.com/2/users/me?user.fields=profile_image_url,url")
        .header(header::ACCEPT, "application/json")
        .bearer_auth(access_token)
        .send()
        .await?
        .text()
        .await?;

    let account_data: XUserResponse = serde_json::from_str(&user_response)?;

    let linked_account = LinkedAccount {
        id: account_data.data.id,
        name: LinkedService::X,
        email: "".to_string(),
        url: format!("https://twitter.com/{}", account_data.data.username),
        avatar: account_data.data.profile_image_url.unwrap_or_default(),
        is_public: true,
        username: account_data.data.username,
        token: None,
        scope: None,
    };

    Ok(linked_account)
}

pub async fn create_linked_in_account(data: AddLinkedAccount) -> error::Result<LinkedAccount> {
    let client = Client::new();
    let client_id = var("LINKEDIN_CLIENT_ID").unwrap();
    let client_secret = var("LINKEDIN_CLIENT_SECRET").unwrap();

    let linkedin_auth = GetLinkedInAccessToken {
        code: data.code,
        client_id: client_id.clone(),
        client_secret,
        grant_type: "authorization_code".to_string(),
        redirect_uri: format!("{}://{}/oauth/callback", PROTOCOL.as_str(), FRONTEND.as_str()),
    };

    let access_response = client
        .post("https://www.linkedin.com/oauth/v2/accessToken")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&linkedin_auth)
        .send()
        .await?
        .text()
        .await?;

    let access_json: LinkedInAccessResponse = serde_json::from_str(&access_response)?;
    let access_token = access_json.access_token;

    let user_response = client
        .get("https://api.linkedin.com/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .text()
        .await?;

    let account_data: LinkedInUserResponse = serde_json::from_str(&user_response)?;

    let linked_account = LinkedAccount {
        id: account_data.sub,
        name: LinkedService::LinkedIn,
        email: account_data.email.unwrap_or_default(),
        url: "".to_string(),
        avatar: account_data.picture.unwrap_or_default(),
        is_public: false,
        username: account_data.name,
        token: None,
        scope: None,
    };

    Ok(linked_account)
}
