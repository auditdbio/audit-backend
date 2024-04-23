use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LinkedService {
    GitHub,
    Gitcoin,
    X,
    LinkedIn,
    WalletConnect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddLinkedAccount {
    pub code: String,
    pub current_role: Option<String>,
    pub service: LinkedService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLinkedAccount {
    pub is_public: Option<bool>,
    pub token: Option<Vec<u8>>,
    pub scope: Option<String>,
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
